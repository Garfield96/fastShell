use std::borrow::{Borrow, BorrowMut};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, Read, Write};
use std::{env, fs, io};

// Build upon https://github.com/ipetkov/conch-parser/blob/master/examples/analysis.rs
use conch_parser::ast;
use conch_parser::ast::{
    Arithmetic, ComplexWord, CompoundCommand, CompoundCommandKind, Parameter,
    ParameterSubstitution, PatternBodyPair, Redirect, RedirectOrEnvVar, SimpleWord,
    TopLevelCommand, TopLevelWord, Word,
};
use conch_parser::lexer::Lexer;
use conch_parser::parse::{DefaultParser, Parser};
use rusqlite::ffi::ErrorCode::FileLockingProtocolFailed;
use rusqlite::{params, AndThenRows, Connection};

use crate::commands::{cat, echo, grep, head, shellCommand, shuf, sort, tail, uniq, wc, Command};
use crate::db_backend::{Postgres, SQLite};
use crate::intermediate::Intermediate;
use crate::STANDALONE;
use conch_parser::ast::builder::StringBuilder;
use postgres::{Client, NoTls};
use rusqlite::config::DbConfig::SQLITE_DBCONFIG_TRIGGER_EQP;
use std::str::Chars;

pub struct Executor<'a> {
    parser: Parser<Lexer<Chars<'a>>, StringBuilder>,
}

impl Executor<'_> {
    pub fn create(script: &String) -> Executor {
        let lex = Lexer::new(script.chars());
        Executor {
            parser: DefaultParser::new(lex),
        }
    }

    pub fn run(self) {
        let mut intermediate: Intermediate = Default::default();
        intermediate.conn = Some(Postgres {
            conn: Some(
                Client::connect(
                    "host=localhost user=postgres password='postgres' dbname='shell'",
                    NoTls,
                )
                .expect("Cannot open connection to DB"),
            ),
            // conn: Some(Connection::open_in_memory().expect("Cannot open connection to DB")),
        });
        intermediate.transaction.push(
            "\
        DROP TABLE IF EXISTS var"
                .to_string(),
        );
        intermediate.transaction.push(
            "\
        CREATE TABLE var (\
        name TEXT UNIQUE NOT NULL,\
        type TEXT,\
        value TEXT)"
                .to_string(),
        );
        for cmd in self.parser {
            eval_cmd(&cmd.unwrap(), intermediate.borrow_mut());
            if !intermediate.sql.is_empty() {
                // intermediate.transaction.push(format!("COPY ({}) TO '/var/lib/postgresql/result.txt'", intermediate.sql));
                intermediate.transaction.push(format!(
                    "COPY ({}) TO '/var/lib/postgresql/result.txt'",
                    intermediate.sql
                ));
                intermediate.sql = String::new();
            }
        }
        let mut sql: String = format!("BEGIN; {} COMMIT;", intermediate.getSQL());
        println!("{}", sql.replace(";", ";\n"));
        intermediate.conn.unwrap().batch_execute(sql).unwrap();
    }
}
fn eval_cmd(cmd: &ast::TopLevelCommand<String>, intermediate: &mut Intermediate) {
    match &cmd.0 {
        ast::Command::Job(list) | ast::Command::List(list) => std::iter::once(&list.first)
            .chain(list.rest.iter().map(|and_or| match and_or {
                ast::AndOr::And(cmd) | ast::AndOr::Or(cmd) => cmd,
            }))
            .for_each(|cmd| {
                eval_listable(&cmd, intermediate);
            }),
    }
}

fn eval_listable(cmd: &ast::DefaultListableCommand, intermediate: &mut Intermediate) {
    match cmd {
        ast::ListableCommand::Single(cmd) => eval_pipeable(intermediate, cmd),
        ast::ListableCommand::Pipe(_, cmds) => cmds
            .into_iter()
            .for_each(|cmd| eval_pipeable(intermediate, cmd)),
    }
}

fn topLevelWordToString(w: &TopLevelWord<String>) -> Option<String> {
    match &w.0 {
        ComplexWord::Concat(_) => None,
        ComplexWord::Single(w) => match w {
            ast::Word::SingleQuoted(w) => Some(w.to_string()),
            ast::Word::Simple(w) => get_simple_word_as_string(w),
            ast::Word::DoubleQuoted(words) if words.len() == 1 => {
                get_simple_word_as_string(&words[0])
            }
            ast::Word::DoubleQuoted(_) => None,
        },
    }
}

fn eval_pipeable(intermediate: &mut Intermediate, cmd: &ast::DefaultPipeableCommand) {
    match cmd {
        ast::PipeableCommand::Simple(cmd) => {
            // println!("Pipeable - Single");
            eval_simple(intermediate, cmd)
        }
        ast::PipeableCommand::Compound(cmd) => {
            match &cmd.kind {
                CompoundCommandKind::Brace(_) => {}
                CompoundCommandKind::Subshell(_) => {}
                CompoundCommandKind::While(w) => {
                    let mut guard_tmp = String::new();
                    for guard in &w.guard {
                        let mut interm: Intermediate = Default::default();
                        eval_cmd(guard, &mut interm);
                    }
                    let mut tmp = String::new();
                    for body in &w.body {
                        let mut interm: Intermediate = Default::default();
                        // body.iter().for_each(|cmd| eval_cmd(cmd, &mut interm));
                        // tmp.push_str(&*format!(
                        //     "WHEN {} THEN {}",
                        //     conditions.join(" or "),
                        //     interm.getSQL()
                        // ));
                    }
                    intermediate.transaction.push(format!(
                        "\
        DO \
        $$ \
        BEGIN \
        WHILE {} LOOP \
        {} \
        END LOOP; \
        END \
        $$",
                        guard_tmp, tmp
                    ));
                }
                CompoundCommandKind::Until(_) => {}
                CompoundCommandKind::If {
                    conditionals,
                    else_branch,
                } => {
                    intermediate.transaction.push(format!(
                        "\
        DO \
        $$ \
        BEGIN \
        IF 0 = 0 THEN \
        COPY data TO '/var/lib/postgresql/13/result2.txt'; \
        ELSE \
        COPY data TO '/var/lib/postgresql/13/result.txt'; \
        END IF; \
        END \
        $$"
                    ));
                }
                CompoundCommandKind::For { .. } => {}
                CompoundCommandKind::Case { word, arms } => {
                    let word_str = topLevelWordToString(word).unwrap();
                    let mut tmp = String::new();
                    for PatternBodyPair { patterns, body } in arms {
                        let conditions: Vec<String> = patterns
                            .iter()
                            .map(topLevelWordToString)
                            .filter(|s| s.is_some())
                            .map(|s| format!("search_str = {}", s.unwrap()))
                            .collect();
                        let mut interm: Intermediate = Default::default();
                        body.iter().for_each(|cmd| eval_cmd(cmd, &mut interm));
                        tmp.push_str(&*format!(
                            "WHEN {} THEN {}",
                            conditions.join(" or "),
                            interm.getSQL()
                        ));
                    }
                    intermediate.transaction.push(format!(
                        "\
        DO \
        $$ \
        DECLARE \
        search_str text; \
        BEGIN \
        search_str := ({}); \
        CASE \
        {} \
        END CASE; \
        END \
        $$",
                        word_str, tmp
                    ));
                }
            };
        }

        ast::PipeableCommand::FunctionDef(_, _) => {
            // println!("Pipeable - Compound");
        }
    };
}

fn eval_simple(intermediate: &mut Intermediate, cmd: &ast::DefaultSimpleCommand) {
    let mut stop = false;
    cmd.redirects_or_env_vars
        .iter()
        .for_each(|redirect_or_var| match redirect_or_var {
            RedirectOrEnvVar::Redirect(_) => {
                panic!("Not supported");
            }
            RedirectOrEnvVar::EnvVar(k, v) => {
                let mut value;
                if let Some(v) = v {
                    value = topLevelWordToString(v).expect("Error parsing value");
                } else {
                    value = "NULL".to_string();
                }
                intermediate.transaction.push(format!(
                    "INSERT INTO var (name, value) VALUES ('{0}',{1}) \
        ON CONFLICT(name) DO UPDATE SET value = {1}",
                    k, value
                ));
                stop = true;
            }
        });
    if stop {
        return;
    }
    let parts: Vec<String> = cmd
        .redirects_or_cmd_words
        .iter()
        .filter_map(|redirect_or_word| match redirect_or_word {
            ast::RedirectOrCmdWord::CmdWord(w) => Some(&w.0),
            ast::RedirectOrCmdWord::Redirect(r) => {
                match r {
                    Redirect::Read(_, _) => {}
                    Redirect::Write(c, w) => {
                        intermediate.redirect = Some(Redirect::Write(*c, String::new()));
                        return Some(&w.0);
                    }
                    Redirect::ReadWrite(_, _) => {}
                    Redirect::Append(c, w) => {
                        intermediate.redirect = Some(Redirect::Append(*c, String::new()));
                        return Some(&w.0);
                    }
                    Redirect::Clobber(_, _) => {}
                    Redirect::Heredoc(_, _) => {}
                    Redirect::DupRead(_, _) => {}
                    Redirect::DupWrite(_, _) => {}
                };
                println!("Redirect not supported");
                None
            }
        })
        .filter_map(|word| match word {
            ast::ComplexWord::Single(w) => Some(w),
            ast::ComplexWord::Concat(c) => None,
        })
        .filter_map(|word| match word {
            ast::Word::SingleQuoted(w) => Some(w.to_string()),
            ast::Word::Simple(w) => get_simple_word_as_string(w.borrow()),

            ast::Word::DoubleQuoted(words) if words.len() == 1 => {
                get_simple_word_as_string(&words[0])
            }
            ast::Word::DoubleQuoted(_) => None,
        })
        // .map(|word| {
        //     println!("{}", word);
        //     word
        // })
        .collect();

    if parts.is_empty() {
        panic!("Empty command");
    }

    if let Some(r) = intermediate.redirect.as_ref() {
        intermediate.redirect = match r {
            Redirect::Write(c, _) => Some(Redirect::Write(*c, parts.last().unwrap().to_string())),
            Redirect::Append(c, _) => Some(Redirect::Append(*c, parts.last().unwrap().to_string())),
            _ => None,
        };
    }
    let parts: Vec<&String> = parts.iter().map(|x| x).collect();
    let mut first = parts.first().unwrap().to_string();
    match first.replace("'", "").as_str() {
        "cat" => {
            <cat::cat as Command>::run(intermediate, parts);
        }
        "head" => {
            <head::head as Command>::run(intermediate, parts);
        }
        "tail" => {
            <tail::tail as Command>::run(intermediate, parts);
        }
        "uniq" => {
            <uniq::uniq as Command>::run(intermediate, parts);
        }
        "sort" => {
            <sort::sort as Command>::run(intermediate, parts);
        }
        "shuf" => {
            <shuf::shuf as Command>::run(intermediate, parts);
        }
        "grep" => {
            <grep::grep as Command>::run(intermediate, parts);
        }
        "wc" => {
            <wc::wc as Command>::run(intermediate, parts);
        }
        "echo" => {
            <echo::echo as Command>::run(intermediate, parts);
        }
        _ => {
            println!("Unknown command: {}", parts.first().unwrap());
            <shellCommand::shellCommand as Command>::run(intermediate, parts);
        }
    };
}

fn get_simple_word_as_string(word: &ast::DefaultSimpleWord) -> Option<String> {
    match word {
        SimpleWord::Literal(w) => Some(match w.as_str() {
            "-le" => "<".to_string(),
            _ => {
                format!("'{}'", w)
            }
        }),
        SimpleWord::Escaped(_) => None,
        SimpleWord::Param(p) => {
            match p {
                Parameter::At => None,
                Parameter::Star => None,
                Parameter::Pound => None,
                Parameter::Question => None,
                Parameter::Dash => None,
                Parameter::Dollar => None,
                Parameter::Bang => None,
                Parameter::Positional(_) => None,
                Parameter::Var(var) => {
                    // TODO: get from table
                    Some(format!("SELECT value FROM var WHERE name = '{}'", var))
                }
            }
        }
        SimpleWord::Subst(_) => None,
        SimpleWord::Star => None,
        SimpleWord::Question => None,
        SimpleWord::SquareOpen => None,
        SimpleWord::SquareClose => None,
        SimpleWord::Tilde => None,
        SimpleWord::Colon => None,
    }
}
