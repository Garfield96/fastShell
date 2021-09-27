use std::borrow::{Borrow, BorrowMut};

// Build upon https://github.com/ipetkov/conch-parser/blob/master/examples/analysis.rs
use conch_parser::ast;
use conch_parser::ast::{
    ComplexWord, CompoundCommandKind, GuardBodyPair, Parameter, PatternBodyPair, Redirect,
    RedirectOrEnvVar, SimpleWord, TopLevelCommand, TopLevelWord,
};
use conch_parser::lexer::Lexer;
use conch_parser::parse::{DefaultParser, Parser};

use crate::commands::{
    cat, echo, grep, head, shellCommand, shuf, sort, tail, test, uniq, wc, Command,
};
use crate::db_backend::Postgres;
use crate::intermediate::{Intermediate, State};

use conch_parser::ast::builder::StringBuilder;
use postgres::{Client, NoTls};

use std::io::Read;
use std::str::Chars;

pub struct Executor<'a> {
    parser: Parser<Lexer<Chars<'a>>, StringBuilder>,
}

enum CommandType {
    Output(String),
    NoOutput(String),
}

impl Executor<'_> {
    pub fn create(script: &str) -> Executor {
        let lex = Lexer::new(script.chars());
        Executor {
            parser: DefaultParser::new(lex),
        }
    }

    pub fn run(self, sql_only: bool) {
        let mut intermediate: Intermediate = Default::default();
        let mut conn = Some(Postgres {
            conn: Some(
                Client::connect(
                    "host=localhost user=postgres password=postgres dbname='shell'",
                    NoTls,
                )
                .expect("Cannot open connection to DB"),
            ),
            // conn: Some(Connection::open_in_memory().expect("Cannot open connection to DB")),
        });

        let batch = self.to_sql(&mut intermediate);

        let mut tx = conn.as_mut().unwrap().transaction().unwrap();
        for cmd in batch {
            match cmd {
                CommandType::Output(cmd) => {
                    let mut sql = cmd;
                    if sql.starts_with("SELECT") {
                        sql = format!("({})", sql);
                    }
                    if !sql_only {
                        sql = format!("COPY {} TO STDOUT;", sql);
                    }
                    sql = sql.replace(";;", ";");
                    println!("{}", sql.replace(";", ";\n"));
                    if !sql_only {
                        let reader = tx.copy_out(sql.as_str()).unwrap();
                        let bytes = reader.bytes().map(|r| r.unwrap()).collect::<Vec<u8>>();
                        println!("\n{}", String::from_utf8(bytes).unwrap());
                    }
                }
                CommandType::NoOutput(cmd) => {
                    let mut sql = cmd.replace(";;", ";");
                    if !sql.contains(' ') {
                        sql = format!("({})", sql);
                    }
                    println!("{}", sql.replace(";", ";\n"));
                    if !sql_only {
                        tx.batch_execute(&*sql).unwrap();
                    }
                }
            }
        }
        tx.commit().unwrap();
    }

    fn to_sql(self, mut intermediate: &mut Intermediate) -> Vec<CommandType> {
        let mut result = Executor::gen_init_code();
        for cmd in self.parser {
            match cmd {
                Ok(cmd) => {
                    eval_cmd(&cmd, intermediate.borrow_mut());
                    for cmd in &intermediate.transaction {
                        result.push(CommandType::NoOutput(cmd.to_string()));
                    }
                    if !intermediate.sql.is_empty() {
                        result.push(CommandType::Output(intermediate.sql.to_string()));
                    }
                    intermediate.transaction.clear();
                    intermediate.sql = String::new();
                }
                Err(e) => {
                    println!("Parser error: {}", e);
                }
            }
        }
        result
    }

    fn gen_init_code() -> Vec<CommandType> {
        vec![CommandType::NoOutput(
            "\
        DROP TABLE IF EXISTS var;
        CREATE TABLE var (\
        name TEXT UNIQUE NOT NULL,\
        type TEXT,\
        value TEXT);"
                .parse()
                .unwrap(),
        )]
    }
}

fn eval_cmd(cmd: &ast::TopLevelCommand<String>, intermediate: &mut Intermediate) {
    match &cmd.0 {
        ast::Command::Job(list) | ast::Command::List(list) => std::iter::once(&list.first)
            .chain(list.rest.iter().map(|and_or| match and_or {
                ast::AndOr::And(cmd) => {
                    // if intermediate.state == State::Condition {
                    //     intermediate.sql.push_str(" && ");
                    // }
                    cmd
                }
                ast::AndOr::Or(cmd) => {
                    // if intermediate.state == State::Condition {
                    //     intermediate.sql.push_str(" || ");
                    // }
                    cmd
                }
            }))
            .for_each(|cmd| {
                eval_listable(&cmd, intermediate);
            }),
    }
}

fn eval_listable(cmd: &ast::DefaultListableCommand, intermediate: &mut Intermediate) {
    match cmd {
        ast::ListableCommand::Single(cmd) => eval_pipeable(intermediate, cmd),
        ast::ListableCommand::Pipe(_, cmds) => {
            cmds.iter().for_each(|cmd| eval_pipeable(intermediate, cmd))
        }
    }
}

fn top_level_word_to_string(w: &TopLevelWord<String>) -> Option<String> {
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
                CompoundCommandKind::Brace(_b) => {
                    println!("Brace")
                }
                CompoundCommandKind::Subshell(_s) => {
                    println!("Subshell")
                }
                CompoundCommandKind::While(w) => {
                    while_stmt(intermediate, &w);
                }
                CompoundCommandKind::Until(w) => {
                    until_stmt(intermediate, &w);
                }
                CompoundCommandKind::If {
                    conditionals: c,
                    else_branch: e,
                } => {
                    if_stmt(intermediate, c, e);
                }
                CompoundCommandKind::For { .. } => {}
                CompoundCommandKind::Case { word, arms } => {
                    case_stmt(intermediate, word, arms);
                }
            };
        }

        ast::PipeableCommand::FunctionDef(_, _) => {
            // println!("Pipeable - FunctionDef");
        }
    };
}

fn if_stmt(
    intermediate: &mut Intermediate,
    c: &[GuardBodyPair<TopLevelCommand<String>>],
    e: &Option<Vec<TopLevelCommand<String>>>,
) {
    let mut eval_cond = String::new();
    let mut eval_body = String::new();
    for cond in c {
        let GuardBodyPair { guard, body } = cond;
        intermediate.state = State::Condition;
        for g in guard {
            eval_cmd(g, intermediate);
        }
        intermediate.state = State::Default;
        eval_cond = intermediate.sql.to_string();
        intermediate.sql = String::new();
        for b in body {
            eval_cmd(b, intermediate);
        }
        eval_body = intermediate.sql.to_string();
        intermediate.sql = String::new();
    }

    let mut else_code = String::new();
    if let Some(e) = e {
        else_code = "ELSE ".to_string();
        for cmd in e {
            intermediate.sql = String::new();
            eval_cmd(cmd, intermediate);
            else_code.push_str(&*format!(
                "INSERT INTO stdout SELECT * FROM ({}) as output;",
                intermediate.sql.to_string()
            ));
            intermediate.sql = String::new();
        }
    }

    intermediate
        .transaction
        .push("DROP TABLE IF EXISTS stdout;".to_string());
    intermediate
        .transaction
        .push("CREATE TABLE stdout (lines TEXT);".to_string());

    intermediate.transaction.push(format!(
        "\
        DO \
        $$ \
        BEGIN \
        IF {} THEN \
        INSERT INTO stdout SELECT * FROM ({}) as output; \
        {}
        END IF; \
        END \
        $$;",
        eval_cond, eval_body, else_code
    ));
    intermediate.sql = "stdout".to_string();
}

fn while_stmt(intermediate: &mut Intermediate, w: &&GuardBodyPair<TopLevelCommand<String>>) {
    let mut eval_guard = String::new();
    let mut eval_body = String::new();
    for guard in &w.guard {
        let mut interm: Intermediate = Default::default();
        eval_cmd(guard, &mut interm);
        eval_guard = interm.sql;
    }
    for body in &w.body {
        let mut interm: Intermediate = Default::default();
        eval_cmd(body, &mut interm);
        eval_body = format!(
            "{}{}",
            eval_body,
            match interm.sql.as_str() {
                "EXIT;" => "EXIT;".to_string(),
                "CONTINUE;" => "CONTINUE;".to_string(),
                s => format!("INSERT INTO stdout SELECT * FROM ({}) as output;", s),
            }
        );
    }
    intermediate
        .transaction
        .push("DROP TABLE IF EXISTS stdout;".to_string());
    intermediate
        .transaction
        .push("CREATE TABLE stdout (lines TEXT);".to_string());
    intermediate.transaction.push(format!(
        "\
        DO \
        $$ \
        BEGIN \
        WHILE {} LOOP \
        {}\
        END LOOP; \
        END \
        $$;",
        eval_guard, eval_body
    ));
    intermediate.sql = "stdout".to_string();
}

fn until_stmt(intermediate: &mut Intermediate, w: &&GuardBodyPair<TopLevelCommand<String>>) {
    let mut eval_guard = String::new();
    let mut eval_body = String::new();
    for guard in &w.guard {
        let mut interm: Intermediate = Default::default();
        eval_cmd(guard, &mut interm);
        eval_guard = interm.sql;
    }
    for body in &w.body {
        let mut interm: Intermediate = Default::default();
        eval_cmd(body, &mut interm);
        eval_body = interm.sql
    }
    intermediate
        .transaction
        .push("DROP TABLE IF EXISTS stdout;".to_string());
    intermediate
        .transaction
        .push("CREATE TABLE stdout (lines TEXT);".to_string());
    intermediate.transaction.push(format!(
        "\
        DO \
        $$ \
        BEGIN \
        LOOP \
        INSERT INTO stdout SELECT * FROM ({}) as output; \
        IF {} THEN
            EXIT;
        END IF;
        END LOOP; \
        END \
        $$;",
        eval_body, eval_guard
    ));
    intermediate.sql = "stdout".to_string();
}

fn case_stmt(
    intermediate: &mut Intermediate,
    word: &TopLevelWord<String>,
    arms: &[PatternBodyPair<TopLevelWord<String>, TopLevelCommand<String>>],
) {
    let word_str = top_level_word_to_string(word).unwrap();
    let word_str = if word_str.contains("SELECT") {
        word_str
    } else {
        format!("'{}'", word_str)
    };
    let mut tmp = String::new();
    for PatternBodyPair { patterns, body } in arms {
        let conditions: Vec<String> = patterns
            .iter()
            .map(top_level_word_to_string)
            .filter(|s| s.is_some())
            .map(|s| format!("'{}'", s.unwrap()))
            .collect();
        let mut interm: Intermediate = Default::default();
        body.iter().for_each(|cmd| eval_cmd(cmd, &mut interm));
        tmp.push_str(&*format!(
            "WHEN {} THEN INSERT INTO stdout SELECT * FROM ({}) as output;",
            conditions.join(", "),
            interm.sql
        ));
    }
    intermediate
        .transaction
        .push("DROP TABLE IF EXISTS stdout;".to_string());
    intermediate
        .transaction
        .push("CREATE TABLE stdout (lines TEXT);".to_string());
    intermediate.transaction.push(format!(
        "\
        DO \
        $$ \
        BEGIN \
        CASE ({}) \
        {} \
        END CASE; \
        END \
        $$",
        word_str, tmp
    ));
    intermediate.sql = "stdout".to_string();
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
                let value;
                if let Some(v) = v {
                    value = top_level_word_to_string(v).expect("Error parsing value");
                } else {
                    value = "NULL".to_string();
                }
                intermediate.transaction.push(format!(
                    "INSERT INTO var (name, value) VALUES ('{0}','{1}') \
                    ON CONFLICT(name) DO UPDATE SET value = '{1}'",
                    k, value
                ));
                stop = true;
            }
        });
    if stop {
        return;
    }
    let mut parts: Vec<String> = cmd
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
            ast::ComplexWord::Concat(_c) => None,
        })
        .filter_map(|word| match word {
            ast::Word::SingleQuoted(w) => Some(w.to_string()),
            ast::Word::Simple(w) => get_simple_word_as_string(w.borrow()),

            ast::Word::DoubleQuoted(words) if words.len() == 1 => {
                get_simple_word_as_string(&words[0]).map(|w| format!("\"{}\"", w))
            }
            ast::Word::DoubleQuoted(_) => None,
        })
        .collect();

    if parts.is_empty() {
        panic!("Empty command");
    }

    if let Some(r) = intermediate.redirect.as_ref() {
        let target = parts.pop().unwrap();
        intermediate.redirect = match r {
            Redirect::Write(c, _) => Some(Redirect::Write(*c, target)),
            Redirect::Append(c, _) => Some(Redirect::Append(*c, target)),
            _ => None,
        };
    }
    let parts: Vec<&String> = parts.iter().collect();
    let first = parts.first().unwrap().to_string();
    match first.replace("'", "").as_str() {
        "cat" => {
            <cat::cat as Command>::run(intermediate, parts);
        }
        "head" => {
            <head::head as Command>::run(intermediate, parts);
        }
        "break" => {
            intermediate.sql = "EXIT;".to_string();
        }
        "continue" => {
            intermediate.sql = "CONTINUE;".to_string();
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
        "test" => {
            <test::test as Command>::run(intermediate, parts);
        }
        "[" => {
            println!("Conditional");
            intermediate.sql = parts
                .iter()
                .skip(1)
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(" ");
        }
        _ => {
            println!("Unknown command: {}", parts.first().unwrap());
            <shellCommand::shellCommand as Command>::run(intermediate, parts);
        }
    };
}

fn get_simple_word_as_string(word: &ast::DefaultSimpleWord) -> Option<String> {
    match word {
        SimpleWord::Literal(w) => Some(w.to_string()),
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
                    Some(format!(
                        "SELECT value FROM var WHERE name = '{}' LIMIT 1",
                        var
                    ))
                }
            }
        }
        SimpleWord::Subst(_) => None,
        SimpleWord::Star => None,
        SimpleWord::Question => None,
        SimpleWord::SquareOpen => Some("test".to_string()),
        SimpleWord::SquareClose => None,
        SimpleWord::Tilde => None,
        SimpleWord::Colon => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::executor::Executor;
    use crate::intermediate::Intermediate;
    use postgres::{Client, NoTls};
    use proptest::prelude::*;

    // TestContext based on https://snoozetime.github.io/2019/06/16/integration-test-diesel.html
    struct TestContext {
        db_name: String,
    }

    impl TestContext {
        fn new(db_name: &str) -> Self {
            let mut client =
                Client::connect("host=localhost user=postgres password=postgres", NoTls).unwrap();
            client
                .execute(format!("DROP DATABASE IF EXISTS {}", db_name).as_str(), &[])
                .unwrap();
            client
                .execute(format!("CREATE DATABASE {}", db_name).as_str(), &[])
                .unwrap();
            Self {
                db_name: db_name.to_string(),
            }
        }
    }

    impl Drop for TestContext {
        fn drop(&mut self) {
            let mut client =
                Client::connect("host=localhost user=postgres password=postgres", NoTls).unwrap();
            // client.execute(format!("DROP DATABASE {}", self.db_name).as_str(), &[]).unwrap();
        }
    }

    #[test]
    fn single_command() {
        let test_db = "single_command_test";
        let _test_ctx = TestContext::new(test_db);

        let script = "cat Cargo.toml";

        let e = Executor::create(script);
        e.run(false);
    }

    #[test]
    fn simple_pipeline() {
        let test_db = "simple_pipeline_test";
        let _test_ctx = TestContext::new(test_db);

        let script = "cat Cargo.toml | shuf";

        let e = Executor::create(script);
        e.run(false);
    }

    #[test]
    fn complex_pipeline() {
        let test_db = "complex_pipeline_test";
        let _test_ctx = TestContext::new(test_db);

        let script = "cat Cargo.toml | shuf | sort -r -b -f | head -n 2 | wc -l";

        let e = Executor::create(script);
        e.run(false);
    }

    #[test]
    fn redirect() {
        let test_db = "redirect_pipeline_test";
        let _test_ctx = TestContext::new(test_db);

        let script = "cat Cargo.toml > result.txt";

        let e = Executor::create(script);
        e.run(false);
    }

    #[test]
    fn case_stmt() {
        let test_db = "case_stmt_test";
        let _test_ctx = TestContext::new(test_db);

        let script = "case \"car2\" in \
           \"car\"|\"car2\") echo \'car\';; \
           \"van\") echo \'van\';; \
        esac;";

        let e = Executor::create(script);
        e.run(false);
    }

    #[test]
    fn case_stmt_var() {
        let test_db = "case_stmt_var_test";
        let _test_ctx = TestContext::new(test_db);

        let script = "\
        var=van; \
        case $var in \
           \"car\") echo \'car\';; \
           \"van\") echo \'van\';; \
        esac;";

        let e = Executor::create(script);
        e.run(false);
    }

    #[test]
    fn if_stmt() {
        let test_db = "if_stmt_test";
        let _test_ctx = TestContext::new(test_db);

        let script = "if [ 1 -le 100 ]; then \
            echo \"1 is less than 100\"; \
            fi";

        let e = Executor::create(script);
        e.run(false);
    }

    #[test]
    fn if_stmt_var() {
        let test_db = "if_stmt_var_test";
        let _test_ctx = TestContext::new(test_db);

        let script = "\
        var='test'; \
        if [ $var -eq 'test' ]; then \
            echo \"var equals test\"; \
        fi";

        let e = Executor::create(script);
        e.run(false);
    }

    #[test]
    fn if_else_stmt() {
        let test_db = "if_else_stmt_test";
        let _test_ctx = TestContext::new(test_db);

        let script = "if [ 1 -ge 100 ]; then \
            echo \'1 is greater than 100\'; \
            else
            echo \'1 is not greater than 100\'; \
            echo \'...\'; \
            fi";

        let e = Executor::create(script);
        e.run(false);
    }

    #[test]
    fn if_test_stmt() {
        let test_db = "if_test_stmt_test";
        let _test_ctx = TestContext::new(test_db);

        let script = "if test 1 -le 100; \
            then \
            echo \"1 is less than 100\"; \
            fi";

        let e = Executor::create(script);
        e.run(false);
    }

    #[test]
    fn while_stmt() {
        let test_db = "while_stmt_test";
        let _test_ctx = TestContext::new(test_db);

        let script = "while test 1 -ne 1; \
            do \
            echo \"1 is less than 100\"; \
            done";

        let e = Executor::create(script);
        e.run(false);
    }

    #[test]
    fn while_exit_stmt() {
        let test_db = "while_exit_stmt_test";
        let _test_ctx = TestContext::new(test_db);

        let script = "while test 1 -eq 1; \
            do \
            break; \
            continue;
            echo \"1 is less than 100\"; \
            done";

        let e = Executor::create(script);
        e.run(false);
    }

    #[test]
    fn until_stmt() {
        let test_db = "until_stmt_test";
        let _test_ctx = TestContext::new(test_db);

        let script = "until test 1 -eq 1; \
            do \
            echo \"1 is less than 100\"; \
            done";

        let e = Executor::create(script);
        e.run(false);
    }

    #[test]
    fn example() {
        let test_db = "example_test";
        let _test_ctx = TestContext::new(test_db);

        let script = "\
        arch='x86'; \
        if [ $arch -eq 'x86' ]; then
            cat test.txt | shuf | sort -r -b -f  | grep 'li' | uniq -c lines | tail -n 2 | head -n 1 | wc -c; \
        else \
            echo 'Not supported'; \
        fi";

        let e = Executor::create(script);
        e.run(false);
    }

    // #[test]
    // fn sqlfs_stmt() {
    //     let test_db = "until_stmt_test";
    //     let _test_ctx = TestContext::new(test_db);
    //
    //     let script = "if [ 1 -ge 100 ]; then \
    //         cat test; \
    //         else
    //         echo \'1 is not greater than 100\'; \
    //         echo \'...\'; \
    //         fi";
    //     let e = Executor::create(script);
    //     e.run(false);
    // }

    proptest! {
        #[test]
        fn test_pipline(script in "cat [a-zA-Z]+ (\\| (shuf))") {
            println!("{}", script);
            let e = Executor::create(script.as_str());
            let mut i : Intermediate = Default::default();
            e.to_sql(&mut i);
        }
    }
}
