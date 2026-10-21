use crate::ast::*;
use crate::lexer::{Lexer, Token};
use crate::messages as msg;

pub struct Parser {
    tokens: Vec<(Token, usize, usize)>,
    pos: usize,
}

type PResult<T> = Result<T, String>;

impl Parser {
    pub fn new(tokens: Vec<(Token, usize, usize)>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_program(&mut self) -> PResult<Block> {
        let mut block = Vec::new();
        self.skip_newlines();
        while !self.check(&Token::Eof) {
            block.push(self.parse_expr()?);
            if self.check(&Token::Eof) { break; }
            if self.check(&Token::Newline) { self.skip_newlines(); }
            else {
                let (l, c) = self.loc();
                return Err(msg::parse_newline_between_exprs(l, c, &format!("{:?}", self.peek())));
            }
        }
        Ok(block)
    }

    fn peek(&self) -> &Token { &self.tokens[self.pos].0 }
    fn loc(&self) -> (usize, usize) { (self.tokens[self.pos].1, self.tokens[self.pos].2) }
    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].0.clone();
        if self.pos < self.tokens.len() - 1 { self.pos += 1; }
        t
    }
    fn check(&self, t: &Token) -> bool { self.peek() == t }
    fn match_tok(&mut self, t: &Token) -> bool {
        if self.check(t) { self.advance(); true } else { false }
    }
    fn expect(&mut self, t: &Token, what: &str) -> PResult<()> {
        if self.match_tok(t) { Ok(()) }
        else {
            let (l, c) = self.loc();
            Err(msg::parse_expect(l, c, what, &format!("{:?}", self.peek())))
        }
    }
    fn expect_end(
        &mut self,
        construct: &str,
        start_line: usize,
        start_col: usize,
    ) -> PResult<()> {
        if self.match_tok(&Token::End) {
            Ok(())
        } else {
            let (l, c) = self.loc();
            Err(msg::parse_expected_end_named(
                construct,
                start_line,
                start_col,
                l,
                c,
                &format!("{:?}", self.peek()),
            ))
        }
    }
    fn skip_newlines(&mut self) {
        while self.check(&Token::Newline) { self.advance(); }
    }

    fn parse_block_until(&mut self, stops: &[Token]) -> PResult<Block> {
        let mut block = Vec::new();
        self.skip_newlines();
        loop {
            if stops.iter().any(|s| self.check(s)) || self.check(&Token::Eof) { break; }
            block.push(self.parse_expr()?);
            if stops.iter().any(|s| self.check(s)) { break; }
            if self.check(&Token::Newline) { self.skip_newlines(); }
            else if !stops.iter().any(|s| self.check(s)) && !self.check(&Token::Eof) {
                let (l, c) = self.loc();
                return Err(msg::parse_newline_between_exprs(l, c, &format!("{:?}", self.peek())));
            }
        }
        Ok(block)
    }

    pub fn parse_expr(&mut self) -> PResult<Expr> {
        if self.check(&Token::Let) { return self.parse_let(); }
        if self.check(&Token::Return) {
            self.advance();
            if self.check(&Token::Newline) || self.check(&Token::End) || self.check(&Token::Eof) {
                return Ok(Expr::Return(None));
            }
            let e = self.parse_expr()?;
            return Ok(Expr::Return(Some(Box::new(e))));
        }
        if self.check(&Token::Break) { self.advance(); return Ok(Expr::Break); }
        if self.check(&Token::Continue) { self.advance(); return Ok(Expr::Continue); }
        if self.check(&Token::Echo) || self.check(&Token::EchoDot) { return self.parse_echo(); }
        self.parse_assign_or_ternary()
    }

    fn parse_let(&mut self) -> PResult<Expr> {
        let (let_line, let_col) = self.loc();
        self.expect(&Token::Let, "ожидалось 'let'")?;
        let mutable = self.match_tok(&Token::Caret);
        if self.check(&Token::LParen) {
            return self.parse_let_tuple(mutable);
        }
        let name = match self.advance() {
            Token::Ident(s) => s,
            other => {
                let (l, c) = self.loc();
                return Err(msg::parse_let_expected_name(l, c, &format!("{:?}", other)));
            }
        };
        if self.check(&Token::LParen) {
            if mutable {
                let (l, c) = self.loc();
                return Err(msg::parse_function_not_mutable(l, c));
            }
            // начало функции — на имени; если нужно, можно на 'let'
            let start = (let_line, let_col);
            let params = self.parse_params()?;
            self.expect(&Token::Eq, &msg::parse_expected_eq_after_params())?;
            let body = self.parse_block_until(&[Token::End])?;
            self.expect_end("let ... (функция)", start.0, start.1)?;
            return Ok(Expr::Let {
                name,
                mutable: false,
                value: Box::new(Expr::Function { params, body }),
            });
        }
        self.expect(&Token::Eq, &msg::parse_expected_eq_after_name())?;
        let value = self.parse_expr()?;
        Ok(Expr::Let { name, mutable, value: Box::new(value) })
    }

    fn parse_let_tuple(&mut self, outer_mutable: bool) -> PResult<Expr> {
        if outer_mutable {
            let (l, c) = self.loc();
            return Err(msg::parse_function_not_mutable(l, c));
        }
        self.expect(&Token::LParen, &msg::parse_expected_lparen())?;
        let mut names = Vec::new();
        loop {
            let m = self.match_tok(&Token::Caret);
            let name = match self.advance() {
                Token::Ident(s) => s,
                other => {
                    let (l, c) = self.loc();
                    return Err(msg::parse_let_expected_name(l, c, &format!("{:?}", other)));
                }
            };
            names.push((name, m));
            if !self.match_tok(&Token::Comma) { break; }
        }
        self.expect(&Token::RParen, &msg::parse_expected_rparen_after_tuple())?;
        self.expect(&Token::Eq, &msg::parse_expected_eq_after_name())?;
        let value = self.parse_expr()?;
        Ok(Expr::LetTuple { names, value: Box::new(value) })
    }

    fn parse_params(&mut self) -> PResult<Vec<Param>> {
        self.expect(&Token::LParen, &msg::parse_expected_lparen())?;
        let mut params: Vec<Param> = Vec::new();
        let mut seen_default = false;
        let mut seen_star = false;
        self.skip_newlines();

        if !self.check(&Token::RParen) {
            loop {
                self.skip_newlines();
                let star = self.match_tok(&Token::Star);
                let name = match self.advance() {
                    Token::Ident(s) => s,
                    other => {
                        let (l, c) = self.loc();
                        return Err(msg::parse_expected_param_name(l, c, &format!("{:?}", other)));
                    }
                };

                let default = if self.match_tok(&Token::Eq) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };

                if seen_star {
                    let (l, c) = self.loc();
                    return Err(msg::parse_star_not_last(l, c, &name));
                }

                if star {
                    if default.is_some() {
                        let (l, c) = self.loc();
                        return Err(msg::parse_star_with_default(l, c, &name));
                    }
                    seen_star = true;
                } else {
                    if default.is_some() {
                        seen_default = true;
                    } else if seen_default {
                        let (l, c) = self.loc();
                        return Err(msg::parse_non_default_after_default(l, c, &name));
                    }
                }

                params.push(Param { name, star, default });

                self.skip_newlines();
                if !self.match_tok(&Token::Comma) { break; }
            }
        }

        self.expect(&Token::RParen, &msg::parse_expected_rparen_after_params())?;
        Ok(params)
    }

    fn parse_echo(&mut self) -> PResult<Expr> {
        let newline = match self.advance() {
            Token::EchoDot => true,
            Token::Echo => false,
            _ => unreachable!(),
        };
        let mut args = Vec::new();
        if self.check(&Token::Newline) || self.check(&Token::End) || self.check(&Token::Eof) {
            let (l, c) = self.loc();
            return Err(msg::parse_echo_no_args(l, c));
        }
        loop {
            args.push(self.parse_expr()?);
            if !self.match_tok(&Token::Comma) { break; }
        }
        Ok(Expr::Echo { args, newline })
    }

    fn parse_assign_or_ternary(&mut self) -> PResult<Expr> {
        if self.check(&Token::Caret)
            && matches!(self.tokens.get(self.pos + 1).map(|t| &t.0), Some(Token::Ident(_)))
        {
            self.advance();
            let name = match self.advance() {
                Token::Ident(s) => s,
                _ => unreachable!(),
            };
            self.expect(&Token::Eq, &msg::parse_expected_eq_after_caret_name())?;
            let value = self.parse_expr()?;
            return Ok(Expr::Assign { name, value: Box::new(value) });
        }
        // Присваивание без ^: Ident = ...
        if let Token::Ident(_) = self.peek() {
            if matches!(self.tokens.get(self.pos + 1).map(|t| &t.0), Some(Token::Eq)) {
                let (l, c) = self.loc();
                return Err(msg::parse_assign_without_caret(l, c));
            }
        }
        self.parse_ternary()
    }

    fn parse_ternary(&mut self) -> PResult<Expr> {
        let expr = self.parse_or()?;
        if self.check(&Token::If) {
            self.advance();
            let cond = self.parse_or()?;
            self.expect(&Token::Else, &msg::parse_ternary_expected_else())?;
    
            // else-ветка парсится как or, а не как ternary:
            // это запрещает цепочку без скобок, но разрешает вложенный тернарник в скобках.
            let else_expr = self.parse_or()?;
    
            // Если после else-выражения снова встретился if — это цепочка без скобок.
            if self.check(&Token::If) {
                let (l, c) = self.loc();
                return Err(msg::parse_ternary_chain_forbidden(l, c));
            }
    
            return Ok(Expr::If {
                branches: vec![(cond, vec![expr])],
                else_branch: Some(vec![else_expr]),
            });
        }
        Ok(expr)
    }

    fn parse_or(&mut self) -> PResult<Expr> {
        let mut left = self.parse_and()?;
        while self.match_tok(&Token::Or) {
            let right = self.parse_and()?;
            left = Expr::Binary { op: BinOp::Or, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> PResult<Expr> {
        let mut left = self.parse_not()?;
        while self.match_tok(&Token::And) {
            let right = self.parse_not()?;
            left = Expr::Binary { op: BinOp::And, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> PResult<Expr> {
        if self.match_tok(&Token::Not) {
            let e = self.parse_not()?;
            return Ok(Expr::Unary { op: UnaryOp::Not, operand: Box::new(e) });
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> PResult<Expr> {
        let mut left = self.parse_concat()?;
        loop {
            let op = match self.peek() {
                Token::EqEq => BinOp::Eq,
                Token::NotEq => BinOp::NotEq,
                Token::Lt => BinOp::Lt,
                Token::LtEq => BinOp::LtEq,
                Token::Gt => BinOp::Gt,
                Token::GtEq => BinOp::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_concat()?;
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_concat(&mut self) -> PResult<Expr> {
        let mut left = self.parse_add()?;
        while self.match_tok(&Token::PlusPlus) {
            let right = self.parse_add()?;
            left = Expr::Binary { op: BinOp::Concat, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> PResult<Expr> {
        let mut left = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_mul()?;
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> PResult<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> PResult<Expr> {
        if self.match_tok(&Token::Minus) {
            let e = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnaryOp::Neg, operand: Box::new(e) });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> PResult<Expr> {
        let mut e = self.parse_primary()?;
        loop {
            if self.match_tok(&Token::LParen) {
                let mut args = Vec::new();
                self.skip_newlines();
                if !self.check(&Token::RParen) {
                    loop {
                        self.skip_newlines();
                        args.push(self.parse_expr()?);
                        self.skip_newlines();
                        if !self.match_tok(&Token::Comma) { break; }
                    }
                }
                self.expect(&Token::RParen, &msg::parse_expected_rparen_after_args())?;
                e = Expr::Call { callee: Box::new(e), args };
            } else if self.match_tok(&Token::LBracket) {
                self.skip_newlines();
                let idx = self.parse_expr()?;
                self.skip_newlines();
                self.expect(&Token::RBracket, &msg::parse_expected_rbracket_after_index())?;
                e = Expr::Index { target: Box::new(e), index: Box::new(idx) };
            } else if self.match_tok(&Token::Dot) {
                let name = match self.advance() {
                    Token::Ident(s) => s,
                    other => {
                        let (l, c) = self.loc();
                        return Err(msg::parse_expected_field_name(l, c, &format!("{:?}", other)));
                    }
                };
                e = Expr::Field { target: Box::new(e), name };
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn parse_primary(&mut self) -> PResult<Expr> {
        let (line, col) = self.loc();
        match self.peek().clone() {
            Token::Number(n) => { self.advance(); Ok(Expr::Number(n)) }
            Token::True => { self.advance(); Ok(Expr::Bool(true)) }
            Token::False => { self.advance(); Ok(Expr::Bool(false)) }
            Token::Nil => { self.advance(); Ok(Expr::Nil) }
            Token::Str(s) => { self.advance(); self.parse_string_token(s) }
            Token::Ident(name) => { self.advance(); Ok(Expr::Var(name)) }
            Token::LParen => { self.advance(); self.parse_paren() }
            Token::LBrace => { self.advance(); self.parse_struct() }
            Token::If => self.parse_if(),
            Token::Match => self.parse_match(),
            Token::Backslash => self.parse_lambda(),
            Token::For => self.parse_for(),
            Token::Loop => self.parse_loop(),
            other => Err(msg::parse_expected_expr(line, col, &format!("{:?}", other))),
        }
    }

    fn parse_string_token(&mut self, s: String) -> PResult<Expr> {
        if !s.contains("$(") {
            return Ok(Expr::Str(s));
        }
        let mut parts: Vec<StrPart> = Vec::new();
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;
        let mut buf = String::new();
        while i < chars.len() {
            if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '(' {
                if !buf.is_empty() { parts.push(StrPart::Lit(std::mem::take(&mut buf))); }
                i += 2;
                let start = i;
                let mut depth = 1usize;
                while i < chars.len() {
                    match chars[i] {
                        '(' => depth += 1,
                        ')' => { depth -= 1; if depth == 0 { break; } }
                        _ => {}
                    }
                    i += 1;
                }
                if i >= chars.len() || depth != 0 {
                    let (l, c) = self.loc();
                    return Err(msg::parse_interp_unclosed(l, c));
                }
                let sub: String = chars[start..i].iter().collect();
                i += 1;
                let mut sub_lexer = Lexer::new(&sub);
                let sub_tokens = sub_lexer.tokenize()
                    .map_err(|e| msg::parse_interp_lex_error(&e))?;
                let mut sub_parser = Parser::new(sub_tokens);
                let e = sub_parser.parse_expr()?;
                if !sub_parser.check(&Token::Eof) {
                    return Err(msg::parse_interp_trailing_tokens());
                }
                parts.push(StrPart::Expr(e));
            } else {
                buf.push(chars[i]);
                i += 1;
            }
        }
        if !buf.is_empty() { parts.push(StrPart::Lit(buf)); }
        Ok(Expr::Interpolated(parts))
    }

    fn parse_paren(&mut self) -> PResult<Expr> {
        self.skip_newlines();
        if self.check(&Token::RParen) {
            self.advance();
            return Ok(Expr::Tuple(vec![]));
        }
        let first = self.parse_expr()?;
        self.skip_newlines();
        if self.match_tok(&Token::Comma) {
            let mut elems = vec![first];
            self.skip_newlines();
            if !self.check(&Token::RParen) {
                loop {
                    self.skip_newlines();
                    elems.push(self.parse_expr()?);
                    self.skip_newlines();
                    if !self.match_tok(&Token::Comma) { break; }
                }
            }
            self.expect(&Token::RParen, &msg::parse_expected_rparen_after_tuple())?;
            Ok(Expr::Tuple(elems))
        } else {
            self.expect(&Token::RParen, &msg::parse_expected_rparen())?;
            Ok(first)
        }
    }

    fn parse_struct(&mut self) -> PResult<Expr> {
        let mut fields = Vec::new();
        self.skip_newlines();
        if !self.check(&Token::RBrace) {
            loop {
                self.skip_newlines();
                let field = if let Token::Ident(_) = self.peek() {
                    if matches!(self.tokens.get(self.pos + 1).map(|t| &t.0), Some(Token::Eq)) {
                        let name = match self.advance() { Token::Ident(s) => s, _ => unreachable!() };
                        self.advance();
                        let v = self.parse_expr()?;
                        StructField { key: Some(name), value: v }
                    } else {
                        StructField { key: None, value: self.parse_expr()? }
                    }
                } else {
                    StructField { key: None, value: self.parse_expr()? }
                };
                fields.push(field);
                self.skip_newlines();
                if !self.match_tok(&Token::Comma) { break; }
            }
        }
        self.expect(&Token::RBrace, &msg::parse_expected_rbrace())?;
        Ok(Expr::Struct(fields))
    }

    fn parse_if(&mut self) -> PResult<Expr> {
        let (start_line, start_col) = self.loc();
        self.expect(&Token::If, &msg::parse_expected_if())?;
        let mut branches = Vec::new();
        let cond = self.parse_expr()?;
        self.expect(&Token::Then, &msg::parse_expected_then_after_if())?;
        let body = self.parse_block_until(&[Token::Elseif, Token::Else, Token::End])?;
        branches.push((cond, body));
        while self.match_tok(&Token::Elseif) {
            let c = self.parse_expr()?;
            self.expect(&Token::Then, &msg::parse_expected_then_after_elseif())?;
            let b = self.parse_block_until(&[Token::Elseif, Token::Else, Token::End])?;
            branches.push((c, b));
        }
        let else_branch = if self.match_tok(&Token::Else) {
            Some(self.parse_block_until(&[Token::End])?)
        } else { None };
        self.expect_end("if", start_line, start_col)?;
        Ok(Expr::If { branches, else_branch })
    }

    fn parse_for(&mut self) -> PResult<Expr> {
        let (start_line, start_col) = self.loc();
        self.expect(&Token::For, &msg::parse_expected_for())?;
    
        if let Token::Ident(name1) = self.peek().clone() {
            match self.tokens.get(self.pos + 1).map(|t| &t.0) {
                Some(Token::In) => {
                    self.advance();
                    self.advance();
                    let iter = self.parse_expr()?;
                    self.expect(&Token::Do, &msg::parse_expected_do_in_for())?;
                    let body = self.parse_block_until(&[Token::End])?;
                    self.expect_end("for", start_line, start_col)?;
                    return Ok(Expr::ForIn { vars: vec![name1], iter: Box::new(iter), body });
                }
                Some(Token::Comma) => {
                    if let Some(Token::Ident(name2)) = self.tokens.get(self.pos + 2).map(|t| t.0.clone()) {
                        if matches!(self.tokens.get(self.pos + 3).map(|t| &t.0), Some(Token::In)) {
                            self.advance();
                            self.advance();
                            self.advance();
                            self.advance();
                            let iter = self.parse_expr()?;
                            self.expect(&Token::Do, &msg::parse_expected_do_in_for())?;
                            let body = self.parse_block_until(&[Token::End])?;
                            self.expect_end("for", start_line, start_col)?;
                            return Ok(Expr::ForIn { vars: vec![name1, name2], iter: Box::new(iter), body });
                        }
                    }
                    let e = self.parse_for_num()?;
                    self.expect_end("for", start_line, start_col)?;
                    return Ok(e);
                }
                _ => {
                    let e = self.parse_for_num()?;
                    self.expect_end("for", start_line, start_col)?;
                    return Ok(e);
                }
            }
        }
        let e = self.parse_for_num()?;
        self.expect_end("for", start_line, start_col)?;
        Ok(e)
    }

    fn parse_for_num(&mut self) -> PResult<Expr> {
        let var = match self.advance() {
            Token::Ident(s) => s,
            other => {
                let (l, c) = self.loc();
                return Err(msg::parse_expected_loop_var_name(l, c, &format!("{:?}", other)));
            }
        };
        self.expect(&Token::Eq, &msg::parse_expected_eq_in_for())?;
        let from = self.parse_expr()?;
        self.expect(&Token::Comma, &msg::parse_expected_comma_in_for())?;
        let to = self.parse_expr()?;
        let step = if self.match_tok(&Token::Comma) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        self.expect(&Token::Do, &msg::parse_expected_do_in_for())?;
        let body = self.parse_block_until(&[Token::End])?;
        Ok(Expr::ForNum { var, from: Box::new(from), to: Box::new(to), step, body })
    }

    fn parse_loop(&mut self) -> PResult<Expr> {
        let (start_line, start_col) = self.loc();
        self.expect(&Token::Loop, &msg::parse_expected_loop())?;
        let body = self.parse_block_until(&[Token::End])?;
        self.expect_end("loop", start_line, start_col)?;
        Ok(Expr::Loop { body })
    }

    fn parse_lambda(&mut self) -> PResult<Expr> {
        let (start_line, start_col) = self.loc();
        self.expect(&Token::Backslash, &msg::parse_expected_backslash())?;
        let params = self.parse_params()?;
        self.expect(&Token::Eq, &msg::parse_expected_eq_after_lambda_params())?;
        let body = self.parse_block_until(&[Token::End])?;
        self.expect_end("лямбда \\", start_line, start_col)?;
        Ok(Expr::Lambda { params, body })
    }

    fn parse_match(&mut self) -> PResult<Expr> {
        let (start_line, start_col) = self.loc();
        self.expect(&Token::Match, &msg::parse_expected_match())?;
        let subject = self.parse_expr()?;
        self.skip_newlines();
        let mut arms = Vec::new();
        while !self.check(&Token::End) && !self.check(&Token::Eof) {
            let pattern = self.parse_pattern()?;
            let guard = if self.match_tok(&Token::If) {
                Some(self.parse_expr()?)
            } else { None };
            self.expect(&Token::Arrow, &msg::parse_expected_arrow_in_match())?;
            let body = self.parse_block_until(&[Token::End, Token::Newline])?;
            arms.push(MatchArm { pattern, guard, body });
            self.skip_newlines();
        }
        self.expect_end("match", start_line, start_col)?;
        Ok(Expr::Match { subject: Box::new(subject), arms })
    }

    fn parse_pattern(&mut self) -> PResult<Pattern> {
        match self.peek().clone() {
            Token::Ident(s) => {
                self.advance();
                if s == "_" { Ok(Pattern::Wildcard) } else { Ok(Pattern::Var(s)) }
            }
            Token::Number(n) => { self.advance(); Ok(Pattern::Number(n)) }
            Token::Str(s) => { self.advance(); Ok(Pattern::Str(s)) }
            Token::True => { self.advance(); Ok(Pattern::Bool(true)) }
            Token::False => { self.advance(); Ok(Pattern::Bool(false)) }
            Token::Nil => { self.advance(); Ok(Pattern::Nil) }
            Token::LParen => {
                self.advance();
                self.skip_newlines();
                let mut pats = Vec::new();
                if !self.check(&Token::RParen) {
                    loop {
                        self.skip_newlines();
                        pats.push(self.parse_pattern()?);
                        self.skip_newlines();
                        if !self.match_tok(&Token::Comma) { break; }
                    }
                }
                self.expect(&Token::RParen, &msg::parse_expected_rparen_in_pattern())?;
                Ok(Pattern::Tuple(pats))
            }
            other => {
                let (l, c) = self.loc();
                Err(msg::parse_expected_pattern(l, c, &format!("{:?}", other)))
            }
        }
    }
}