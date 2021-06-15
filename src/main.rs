// Build upon https://github.com/ipetkov/conch-parser/blob/master/examples/analysis.rs
use conch_parser::ast;
use conch_parser::ast::{
    Arithmetic, Parameter, ParameterSubstitution, Redirect, SimpleWord, TopLevelCommand,
    TopLevelWord, Word,
};
use conch_parser::lexer::Lexer;
use conch_parser::parse::DefaultParser;
use rusqlite::{params, Connection};
use std::borrow::Borrow;
use std::fs::File;
use std::io::{BufRead, Write};
use std::{env, fs, io};

mod commands;
use crate::commands::{cat, grep, head, shellCommand, shuf, sort, tail, uniq, wc, Command};

mod getxattr;

const STANDALONE: bool = true;

#[derive(Debug)]
struct Intermediate {
    sql: String,
    conn: Option<rusqlite::Connection>,
    redirect: bool,
    redirect_target: String,
}

impl Default for Intermediate {
    fn default() -> Self {
        Intermediate {
            sql: "".to_string(),
            conn: None,
            redirect: false,
            redirect_target: "".to_string(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    let lex = Lexer::new(args.first().unwrap().chars());
    let parser = DefaultParser::new(lex);

    for cmd in parser {
        eval_cmd(&cmd?);
    }
    Ok(())
}

fn eval_cmd(cmd: &ast::TopLevelCommand<String>) {
    match &cmd.0 {
        ast::Command::Job(list) | ast::Command::List(list) => std::iter::once(&list.first)
            .chain(list.rest.iter().map(|and_or| match and_or {
                ast::AndOr::And(cmd) | ast::AndOr::Or(cmd) => cmd,
            }))
            .for_each(|cmd| {
                let mut r = eval_listable(&cmd);
                println!("SQL: {}", r.sql);

                if STANDALONE {
                    let conn = r.conn.unwrap();
                    let stmt = conn.prepare(r.sql.as_str());
                    let r_vec: Vec<String> = stmt
                        .unwrap()
                        .query_map([], |entry| entry.get::<_, String>(0))
                        .unwrap()
                        .into_iter()
                        .map(|s| s.unwrap())
                        .collect();
                    let result_string = r_vec.join("\n");
                    if r.redirect {
                        let mut f = File::create(r.redirect_target).unwrap();
                        f.write_all(result_string.as_bytes());
                    }
                    println!("{}", result_string);
                } else {
                    if r.redirect {
                        getxattr::getxattr(
                            ".",
                            format!(
                                "sql#CREATE TABLE {} (lines TEXT NOT NULL)",
                                r.redirect_target
                            ),
                            false,
                        )
                        .unwrap();
                        r.sql = format!("INSERT INTO {} {}", r.redirect_target, r.sql);
                    }
                    let r = getxattr::getxattr(".", format!("sql#{}", r.sql), false).unwrap();
                    println!("{}", String::from_utf8(r).unwrap());
                }
            }),
    }
}

fn eval_listable(cmd: &ast::DefaultListableCommand) -> Intermediate {
    // println!("Sub command");
    match cmd {
        ast::ListableCommand::Single(cmd) => eval_pipeable(Default::default(), cmd),
        ast::ListableCommand::Pipe(_, cmds) => cmds
            .into_iter()
            .fold(Default::default(), |i, cmd| eval_pipeable(i, cmd)),
    }
}

fn eval_pipeable(intermediate: Intermediate, cmd: &ast::DefaultPipeableCommand) -> Intermediate {
    match cmd {
        ast::PipeableCommand::Simple(cmd) => {
            // println!("Pipeable - Single");
            eval_simple(intermediate, cmd)
        }
        ast::PipeableCommand::Compound(_) | ast::PipeableCommand::FunctionDef(_, _) => {
            // println!("Pipeable - Compound");
            Default::default()
        }
    }
}

fn eval_simple(mut intermediate: Intermediate, cmd: &ast::DefaultSimpleCommand) -> Intermediate {
    let parts: Vec<&String> = cmd
        .redirects_or_cmd_words
        .iter()
        .filter_map(|redirect_or_word| match redirect_or_word {
            ast::RedirectOrCmdWord::CmdWord(w) => Some(&w.0),
            ast::RedirectOrCmdWord::Redirect(r) => {
                match r {
                    Redirect::Read(_, _) => {}
                    Redirect::Write(_, w) => {
                        intermediate.redirect = true;
                        return Some(&w.0);
                    }
                    Redirect::ReadWrite(_, _) => {}
                    Redirect::Append(_, _) => {}
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
            ast::ComplexWord::Concat(_) => None,
        })
        .filter_map(|word| match word {
            ast::Word::SingleQuoted(w) => Some(w),
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

    if intermediate.redirect {
        intermediate.redirect_target = parts.last().unwrap().to_string();
    }
    match parts.first().unwrap().as_str() {
        "cat" => {
            <cat::cat as Command>::run(&mut intermediate, parts);
        }
        "head" => {
            <head::head as Command>::run(&mut intermediate, parts);
        }
        "tail" => {
            <tail::tail as Command>::run(&mut intermediate, parts);
        }
        "uniq" => {
            <uniq::uniq as Command>::run(&mut intermediate, parts);
        }
        "sort" => {
            <sort::sort as Command>::run(&mut intermediate, parts);
        }
        "shuf" => {
            <shuf::shuf as Command>::run(&mut intermediate, parts);
        }
        "grep" => {
            <grep::grep as Command>::run(&mut intermediate, parts);
        }
        "wc" => {
            <wc::wc as Command>::run(&mut intermediate, parts);
        }
        _ => {
            println!("Unknown command: {}", parts.first().unwrap());
            <shellCommand::shellCommand as Command>::run(&mut intermediate, parts);
        }
    };
    intermediate
}

fn get_simple_word_as_string(word: &ast::DefaultSimpleWord) -> Option<&String> {
    match word {
        ast::SimpleWord::Literal(w) => Some(w),
        _ => None,
    }
}
