#[cfg(test)]
mod helper {
    use crate::eval_apply::eval;
    use crate::parser::{InPort, TOKENIZER};
    use crate::prelude::{get_prelude, make_env_ptr};
    use crate::types::*;
    use std::error::Error;
    use std::rc::Rc;

    struct MockInput<'a> {
        line: String,
        lines: std::str::Lines<'a>,
    }

    impl<'a> MockInput<'a> {
        fn new(input: &'a str) -> MockInput<'a> {
            MockInput {
                line: String::new(),
                lines: input.lines(),
            }
        }
    }

    impl<'a> InPort for MockInput<'a> {
        fn readline(&mut self) -> Option<Result<String, Box<dyn Error>>> {
            match self.lines.next() {
                Some(line) => (Some(Ok(line.into()))),
                None => None,
            }
        }

        fn next_token(&mut self) -> Option<Result<String, Box<dyn Error>>> {
            loop {
                if &self.line == "" {
                    self.line = match self.readline() {
                        Some(Ok(line)) => line,
                        None => String::new(),
                        _ => unreachable!(),
                    };
                }
                if &self.line == "" {
                    return None;
                } else {
                    let next = TOKENIZER.captures_iter(&self.line).next();
                    let (token, rest) = match next {
                        Some(cap) => (String::from(&cap[1]), String::from(&cap[2])),
                        None => unreachable!(),
                    };
                    self.line = rest;
                    match token.chars().nth(0) {
                        Some(';') | None => (),
                        _ => return Some(Ok(token.into())),
                    };
                }
            }
        }
    }

    fn check_io_str(input: &str, output: &str, env: &RcRefCell<Env>) {
        // let str_exp = input.to_string();
        let mut mock = MockInput::new(input);
        let right = output.to_string();
        let left = match mock.read_next_exp() {
            Ok(exp) => {
                let val = eval(exp, Rc::clone(env));
                format!("{:?}", val)
            }
            Err(e) => format!("Error: {:?}", e),
        };
        assert_eq!(left, right);
    }

    pub fn check_io(pairs: Vec<(&str, &str)>) {
        let env = make_env_ptr(get_prelude());
        pairs.iter().for_each(|(i, o)| check_io_str(i, o, &env));
    }
}

#[cfg(test)]
mod basics {
    use super::helper::check_io;

    #[test]
    fn plus_simple() {
        check_io(vec![("(+ 1 2)", "Ok(3)")]);
    }

    #[test]
    fn plus_nested() {
        check_io(vec![("(+ 1 (* 2 3))", "Ok(7)")]);
    }

    #[test]
    fn quote() {
        check_io(vec![("(quote (1 2 3))", "Ok([1, 2, 3])")]);
    }

    #[test]
    fn define_val() {
        check_io(vec![
            ("(define x 3)", "Ok()"),
            ("x", "Ok(3)"),
            ("(+ x 1)", "Ok(4)"),
        ]);
    }

    #[test]
    fn define_proc_basic() {
        check_io(vec![
            ("(define x 3)", "Ok()"),
            ("x", "Ok(3)"),
            ("(define one (lambda () 1))", "Ok()"),
            ("(one)", "Ok(1)"),
            ("(+ (one) (+ 2 x))", "Ok(6)"),
        ]);
    }

    #[test]
    fn define_proc_call_prim() {
        check_io(vec![
            ("(define x 3)", "Ok()"),
            ("x", "Ok(3)"),
            ("(define inc (lambda (x) (+ x 1)))", "Ok()"),
            ("(inc 100)", "Ok(101)"),
            ("(inc x)", "Ok(4)"),
        ]);
    }

    #[test]
    fn cond() {
        check_io(vec![
            ("(if #t 123 wtf)", "Ok(123)"),
            ("(if #f wtf 123)", "Ok(123)"),
            ("(cond (#f wtf0) (#f wtf1) (#t 456) (else wtf3))", "Ok(456)"),
            ("(cond (#f wtf0) (#f wtf1) (#f wtf2) (else 789))", "Ok(789)"),
        ]);
    }

    #[test]
    fn eq() {
        check_io(vec![
            ("(define one (lambda () 1))", "Ok()"),
            ("(= 1 1)", "Ok(true)"),
            ("(= 1 (one))", "Ok(true)"),
            ("(if (= 1 (one)) 123 wtf)", "Ok(123)"),
            ("(if (= (one) (+ 4 5)) wtf 123)", "Ok(123)"),
        ]);
    }

    #[test]
    fn cons_car_cdr() {
        check_io(vec![
            ("(car (cons 123 456))", "Ok(123)"),
            ("(cdr (cons 123 456))", "Ok(456)"),
            ("(define p (cons (cons 1 2) (cons 3 4)))", "Ok()"),
            ("(cdr (car p))", "Ok(2)"),
            ("(cdr p)", "Ok([3, 4])"),
            ("p", "Ok([[1, 2], [3, 4]])"),
            ("(define l (cons 1 (cons 2 (cons 3 null))))", "Ok()"),
            ("(car (cdr l))", "Ok(2)"),
            ("(cdr (cdr (cdr l)))", "Ok([])"),
        ]);
    }

    #[test]
    fn begin() {
        check_io(vec![(
            "(begin (define one (lambda () 1)) (+ (one) 2))",
            "Ok(3)",
        )]);
    }

    #[test]
    fn multiline_simple() {
        check_io(vec![(
            "(begin
                (define one
                    (lambda () 1))
                (+ (one) 2))",
            "Ok(3)",
        )]);
    }

    #[test]
    fn multiline_comment() {
        check_io(vec![(
            "(begin
                (define one ; something here
                    ; generating the number 1
                    ;; more quotes
                    (lambda () 1))
                (+ (one) 2))",
            "Ok(3)",
        )]);
    }

    #[test]
    fn inline_lambda() {
        check_io(vec![(
            "((lambda (x y z)
                    (+ x
                       (+ y z))) 1
                                 2
                                 3)",
            "Ok(6)",
        )]);
    }
}

#[cfg(test)]
mod sugar {
    use super::helper::check_io;

    #[test]
    fn sugar_lambda() {
        check_io(vec![(
            "((lambda (x y z)
                    (quote whatever)
                    (+ x
                       (+ y z))) 1
                                 2
                                 3)",
            "Ok(6)",
        )]);
    }

    #[test]
    fn sugar_define_definition() {
        check_io(vec![
            (
                "(define (add3 x y z)
                    (+ x
                       (+ y z)))",
                "Ok()",
            ),
            (
                "(add3 101 
                       102 
                       103))",
                "Ok(306)",
            ),
        ]);
    }

    #[test]
    fn sugar_define_body() {
        check_io(vec![
            (
                "(define (three)
                    (quote whatever)
                    (define one (lambda () 1))
                    (+ (one) 2))",
                "Ok()",
            ),
            ("(three)", "Ok(3)"),
        ]);
    }
}

#[cfg(test)]
mod environment {
    use super::helper::check_io;

    #[test]
    fn set_simple() {
        check_io(vec![
            ("(define inc (lambda (x) (+ x 1)))", "Ok()"),
            ("(define x 3)", "Ok()"),
            ("(set! x (inc x))", "Ok()"),
            ("x", "Ok(4)"),
            ("(set! x (inc x))", "Ok()"),
            ("x", "Ok(5)"),
        ]);
    }

    #[test]
    fn set_bank_account() {
        check_io(vec![
            (
                "(define account
                    (lambda (bal)
                        (lambda (amt)
                            (begin 
                                (set! bal (+ bal amt)) 
                                bal))))",
                "Ok()",
            ),
            ("(define a1 (account 100))", "Ok()"),
            ("(a1 0)", "Ok(100)"),
            ("(a1 10)", "Ok(110)"),
            ("(a1 10)", "Ok(120)"),
        ]);
    }
}

#[cfg(test)]
mod general {
    use super::helper::check_io;

    #[test]
    fn sqrt_200() {
        check_io(vec![
            ("(define (abs x) (if (>= x 0) x (- 0 x)))", "Ok()"),
            (
                "(define (newton guess function derivative epsilon)
                    (define guess2 (- guess (/ (function guess) (derivative guess))))
                    (if (< (abs (- guess guess2)) epsilon) guess2
                        (newton guess2 function derivative epsilon)))",
                "Ok()",
            ),
            (
                "(define (square-root a)
                    (newton 1 (lambda (x) (- (* x x) a)) (lambda (x) (* 2 x)) 1e-8))",
                "Ok()",
            ),
            ("(> (square-root 200) 14.14213)", "Ok(true)"),
            ("(< (square-root 200) 14.14215)", "Ok(true)"),
        ]);
    }

    #[test]
    fn fibonacci() {
        check_io(vec![
            (
                "(define fib (lambda (n) (if (< n 2) 1 (+ (fib (- n 1)) (fib (- n 2))))))",
                "Ok()",
            ),
            (
                "(fib 20)", 
                "Ok(10946)"
            ),
            (
                "(define range (lambda (a b) (if (= a b) (quote ()) (cons a (range (+ a 1) b)))))",
                "Ok()",
            ),
            (
                "(define map (lambda (f l) (if (null? l) null (cons (f (car l)) (map f (cdr l))))))",
                "Ok()",
            ),
            (
                "(range 0 10)",
                "Ok([0, [1, [2, [3, [4, [5, [6, [7, [8, [9, []]]]]]]]]]])",
            ),
            (
                "(map fib (range 0 10))",
                "Ok([1, [1, [2, [3, [5, [8, [13, [21, [34, [55, \
                 []]]]]]]]]]])",
            ),
        ]);
    }

    #[test]
    #[ignore]
    fn fibonacci_long() {
        check_io(vec![
            (
                "(define fib (lambda (n) (if (< n 2) 1 (+ (fib (- n 1)) (fib (- n 2))))))",
                "Ok()",
            ),
            (
                "(define range (lambda (a b) (if (= a b) (quote ()) (cons a (range (+ a 1) b)))))",
                "Ok()",
            ),
            (
                "(define map (lambda (f l) (if (null? l) null (cons (f (car l)) (map f (cdr l))))))",
                "Ok()",
            ),
            (
                "(map fib (range 0 20))",
                "Ok([1, [1, [2, [3, [5, [8, [13, [21, [34, [55, [89, [144, \
                 [233, [377, [610, [987, [1597, [2584, [4181, [6765, \
                 []]]]]]]]]]]]]]]]]]]]])",
            ),
        ]);
    }

    #[test]
    fn man_or_boy() {
        // ! Doesn't work. Looks related to the lambda -> lambda-begin expansion
        check_io(vec![
            (
                "(define A (lambda (k x1 x2 x3 x4 x5) 
                    (define B (lambda () (set! k (- k 1)) (A k B x1 x2 x3 x4))) 
                    (if (<= k 0) (+ (x4) (x5)) (B))))",
                "Ok()",
            ),
            (
                "(A 10 (lambda () 1) (lambda () -1) (lambda () -1) (lambda () 1) (lambda () 0))",
                "Ok(-67)",
            ),
        ]);
    }
}
