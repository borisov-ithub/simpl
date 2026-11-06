use crate::ast::*;
use crate::lexer::{Lexer, Token};
use crate::messages as msg;

pub struct Parser {
    tokens: Vec<(Token, usize, usize)>,
    pos: usize,
    in_struct: usize,
}

type PResult<T> = Result<T, String>;

impl Parser {
    pub fn new(tokens: Vec<(Token, usize, usize)>) -> Self {
        Self { tokens, pos: 0, in_struct: 0 }
    }

    pub fn parse_program(&mut self) -> PResult<Block> {
        let mut block = Vec::new();
        self.skip_newlines();
        while !self.check(&Token::Eof) {
            if self.check(&Token::Use) {
                block.push(self.parse_use()?);
            } else if self.check(&Token::Import) {
                block.push(self.parse_import()?);
            } else {
                block.push(self.parse_expr()?);
            }
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
    fn skip_newlines(&mut self) {
        while self.check(&Token::Newline) { self.advance(); }
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

    fn parse_block_until(&mut self, stops: &[Token]) -> PResult<Block> {
        let saved_in_struct = self.in_struct;
        self.in_struct = 0;
        let result = self.parse_block_until_inner(stops);
        self.in_struct = saved_in_struct;
        result
    }
    
    fn parse_block_until_inner(&mut self, stops: &[Token]) -> PResult<Block> {
        let mut block = Vec::new();
        self.skip_newlines();
        loop {
            if stops.iter().any(|s| self.check(s)) || self.check(&Token::Eof) { break; }
            if self.check(&Token::Use) {
                let (l, c) = self.loc();
                return Err(msg::parse_use_only_top_level(l, c));
            }
            if self.check(&Token::Import) {
                let (l, c) = self.loc();
                return Err(msg::parse_import_only_top_level(l, c));
            }
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

    fn parse_use(&mut self) -> PResult<Expr> {
        let (line, col) = self.loc();
        self.expect(&Token::Use, "ожидалось 'use'")?;
        let module = match self.advance() {
            Token::Ident(s) => s,
            other => return Err(msg::parse_use_expected_module(line, col, &format!("{:?}", other))),
        };
        let alias = if self.match_tok(&Token::As) {
            match self.advance() {
                Token::Ident(s) => Some(s),
                other => return Err(msg::parse_use_expected_module(line, col, &format!("{:?}", other))),
            }
        } else {
            None
        };
        let names = if self.match_tok(&Token::LParen) {
            if self.match_tok(&Token::Star) {
                self.expect(&Token::RParen, &msg::parse_use_expected_rparen())?;
                UseNames::Wildcard
            } else {
                let mut list = Vec::new();
                loop {
                    match self.advance() {
                        Token::Ident(s) => list.push(s),
                        other => {
                            let (l, c) = self.loc();
                            return Err(msg::parse_use_expected_name(l, c, &format!("{:?}", other)));
                        }
                    }
                    if !self.match_tok(&Token::Comma) { break; }
                }
                self.expect(&Token::RParen, &msg::parse_use_expected_rparen())?;
                UseNames::Explicit(list)
            }
        } else if alias.is_some() {
            UseNames::AliasOnly
        } else {
            return Err(msg::parse_use_requires_names(&module));
        };
        Ok(Expr::Use { module, alias, names })
    }

    fn parse_import(&mut self) -> PResult<Expr> {
        let (line, col) = self.loc();
        self.expect(&Token::Import, "ожидалось 'import'")?;
        let path = self.parse_import_path()?;
    
        let explicit_alias = if self.match_tok(&Token::As) {
            match self.advance() {
                Token::Ident(s) => Some(s),
                other => return Err(msg::parse_import_expected_path(line, col, &format!("{:?}", other))),
            }
        } else {
            None
        };
    
        let names = if self.match_tok(&Token::LParen) {
            if self.match_tok(&Token::Star) {
                self.expect(&Token::RParen, &msg::parse_use_expected_rparen())?;
                UseNames::Wildcard
            } else {
                let mut list = Vec::new();
                loop {
                    match self.advance() {
                        Token::Ident(s) => list.push(s),
                        other => {
                            let (l, c) = self.loc();
                            return Err(msg::parse_use_expected_name(l, c, &format!("{:?}", other)));
                        }
                    }
                    if !self.match_tok(&Token::Comma) { break; }
                }
                self.expect(&Token::RParen, &msg::parse_use_expected_rparen())?;
                UseNames::Explicit(list)
            }
        } else if explicit_alias.is_some() {
            UseNames::AliasOnly
        } else {
            // import utils — модуль под своим именем
            UseNames::AliasOnly
        };
    
        // Если алиас не задан явно, но names == AliasOnly — берём последний сегмент пути
        let alias = match (explicit_alias, &names) {
            (Some(a), _) => Some(a),
            (None, UseNames::AliasOnly) => {
                // "library/utils" → "utils"; "../file" → "file"; "utils.spl" → "utils"
                let name = path
                    .rsplit('/')
                    .next()
                    .unwrap_or(&path)
                    .trim_end_matches(".spl")
                    .to_string();
                Some(name)
            }
            (None, _) => None,
        };
    
        Ok(Expr::Import { path, alias, names })
    }
    
    fn parse_import_path(&mut self) -> PResult<String> {
        // Строка в кавычках — берём как есть
        if let Token::Str(s) = self.peek() {
            let s = s.clone();
            self.advance();
            return Ok(s);
        }
    
        let mut parts: Vec<String> = Vec::new();
    
        // Первый сегмент: Ident, DotDot (..), Dot (.)
        match self.peek() {
            Token::DotDot => {
                self.advance();
                parts.push("..".to_string());
            }
            Token::Dot => {
                self.advance();
                parts.push(".".to_string());
            }
            Token::Ident(_) => {
                let s = match self.advance() {
                    Token::Ident(s) => s,
                    _ => unreachable!(),
                };
                parts.push(s);
            }
            other => {
                let (l, c) = self.loc();
                return Err(msg::parse_import_expected_path(l, c, &format!("{:?}", other)));
            }
        }
        
        // Остальные сегменты через "/"
        while self.match_tok(&Token::Slash) {
            match self.advance() {
                Token::Ident(s) => parts.push(s),
                other => {
                    let (l, c) = self.loc();
                    return Err(msg::parse_import_expected_segment(l, c, &format!("{:?}", other)));
                }
            }
        }
    
        Ok(parts.join("/"))
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
                let (name_line, name_col) = self.loc();
                let name = match self.advance() {
                    Token::Ident(s) => s,
                    other => {
                        let (l, c) = self.loc();
                        return Err(msg::parse_expected_param_name(l, c, &format!("{:?}", other)));
                    }
                };

                // проверка дубликата
                if params.iter().any(|p| p.name == name) {
                    return Err(msg::parse_duplicate_param(name_line, name_col, &name));
                }

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
        // ^name = expr — присваивание переменной (только вне struct-литерала)
        if self.in_struct == 0
            && self.check(&Token::Caret)
            && matches!(self.tokens.get(self.pos + 1).map(|t| &t.0), Some(Token::Ident(_)))
            && matches!(self.tokens.get(self.pos + 2).map(|t| &t.0), Some(Token::Eq))
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

        // name = expr — ошибка (нужен ^), только вне struct-литерала
        if self.in_struct == 0 {
            if let Token::Ident(_) = self.peek() {
                if matches!(self.tokens.get(self.pos + 1).map(|t| &t.0), Some(Token::Eq)) {
                    let (l, c) = self.loc();
                    return Err(msg::parse_assign_without_caret(l, c));
                }
            }
        }

        let (expr_line, expr_col) = self.loc();
        let expr = self.parse_ternary()?;

        // Присваивание полю/индексу — только вне struct-литерала
        if self.in_struct == 0 && self.match_tok(&Token::Eq) {
            let value = self.parse_expr()?;
            let (target, key) = decompose_assign_target(expr)
                .ok_or_else(|| msg::parse_bad_assign_target(expr_line, expr_col))?;
            return Ok(Expr::AssignField { target, key, value: Box::new(value) });
        }

        Ok(expr)
    }

    fn parse_pipe(&mut self) -> PResult<Expr> {
        let mut left = self.parse_or()?;
        while self.match_tok(&Token::Dollar) {
            let right = self.parse_or()?;
            left = Expr::Pipe { left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_ternary(&mut self) -> PResult<Expr> {
        let expr = self.parse_pipe()?;
        if self.check(&Token::If) {
            self.advance();
            let cond = self.parse_pipe()?;
            self.expect(&Token::Else, &msg::parse_ternary_expected_else())?;
            let else_expr = self.parse_pipe()?;
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
        let mut left = self.parse_bit_or()?;
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
            let right = self.parse_bit_or()?;
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }
    
    fn parse_bit_or(&mut self) -> PResult<Expr> {
        let mut left = self.parse_bit_xor()?;
        while self.match_tok(&Token::Pipe) {
            let right = self.parse_bit_xor()?;
            left = Expr::Binary { op: BinOp::BitOr, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }
    
    fn parse_bit_xor(&mut self) -> PResult<Expr> {
        let mut left = self.parse_bit_and()?;
        while self.match_tok(&Token::Xor) {
            let right = self.parse_bit_and()?;
            left = Expr::Binary { op: BinOp::BitXor, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }
    
    fn parse_bit_and(&mut self) -> PResult<Expr> {
        let mut left = self.parse_shift()?;
        while self.match_tok(&Token::Amp) {
            let right = self.parse_shift()?;
            left = Expr::Binary { op: BinOp::BitAnd, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }
    
    fn parse_shift(&mut self) -> PResult<Expr> {
        let mut left = self.parse_concat()?;
        loop {
            let op = match self.peek() {
                Token::Shl => BinOp::Shl,
                Token::Shr => BinOp::Shr,
                _ => break,
            };
            self.advance();
            let right = self.parse_concat()?;
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_concat(&mut self) -> PResult<Expr> {
        // обрабатывает и ++, и ..
        let mut left = self.parse_add()?;
        loop {
            if self.match_tok(&Token::PlusPlus) {
                let right = self.parse_add()?;
                left = Expr::Binary { op: BinOp::Concat, left: Box::new(left), right: Box::new(right) };
            } else if self.match_tok(&Token::DotDot) {
                let right = self.parse_add()?;
                left = Expr::Compose { left: Box::new(left), right: Box::new(right) };
            } else {
                break;
            }
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
        if self.match_tok(&Token::Tilde) {
            let e = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnaryOp::BitNot, operand: Box::new(e) });
        }
        if self.match_tok(&Token::Type) {
            let e = self.parse_unary()?;
            return Ok(Expr::TypeOf(Box::new(e)));
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
            Token::Use => Err(msg::parse_use_only_top_level(line, col)),
            Token::Caret => {
                self.advance(); // ^
                match self.advance() {
                    Token::Ident(s) => Ok(Expr::Var(s)),
                    other => {
                        let (l, c) = self.loc();
                        Err(msg::parse_expected_expr(l, c, &format!("{:?}", other)))
                    }
                }
            }
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
        self.in_struct += 1;
        let result = self.parse_struct_inner();
        self.in_struct -= 1;
        result
    }

    fn parse_struct_inner(&mut self) -> PResult<Expr> {
        self.skip_newlines();
        if self.check(&Token::RBrace) {
            self.advance();
            return Ok(Expr::Struct(vec![]));
        }
    
        // Первый expr — может быть ключом comprehension или полем/автоключом
        let first = self.parse_expr()?;
    
        // Если после первого expr идёт ':' — это comprehension
        if self.match_tok(&Token::Colon) {
            return self.parse_comp_after_colon(first);
        }
    
        // Иначе — обычный литерал (как раньше)
        let mut fields = Vec::new();
        let field = if self.match_tok(&Token::Eq) {
            let (l, c) = self.loc();
            let key = expr_to_struct_key(&first)
                .ok_or_else(|| msg::parse_bad_struct_key(l, c))?;
            let v = self.parse_expr()?;
            StructField { key: Some(key), value: v }
        } else {
            StructField { key: None, value: first }
        };
        fields.push(field);
    
        self.skip_newlines();
        while self.match_tok(&Token::Comma) {
            self.skip_newlines();
            if self.check(&Token::RBrace) { break; }
            let e = self.parse_expr()?;
            let field = if self.match_tok(&Token::Eq) {
                let (l, c) = self.loc();
                let key = expr_to_struct_key(&e)
                    .ok_or_else(|| msg::parse_bad_struct_key(l, c))?;
                let v = self.parse_expr()?;
                StructField { key: Some(key), value: v }
            } else {
                StructField { key: None, value: e }
            };
            fields.push(field);
            self.skip_newlines();
        }
    
        self.expect(&Token::RBrace, &msg::parse_expected_rbrace())?;
        Ok(Expr::Struct(fields))
    }
    
    fn parse_comp_after_colon(&mut self, key: Expr) -> PResult<Expr> {
        let value = self.parse_expr()?;
    
        // Ожидаем 'for'
        if !self.match_tok(&Token::For) {
            let (l, c) = self.loc();
            return Err(msg::parse_comp_expected_for(l, c, &format!("{:?}", self.peek())));
        }
    
        // Переменные: одна или две через запятую
        let mut vars = Vec::new();
        match self.advance() {
            Token::Ident(s) => vars.push(s),
            other => {
                let (l, c) = self.loc();
                return Err(msg::parse_comp_expected_var(l, c, &format!("{:?}", other)));
            }
        }
        if self.match_tok(&Token::Comma) {
            match self.advance() {
                Token::Ident(s) => vars.push(s),
                other => {
                    let (l, c) = self.loc();
                    return Err(msg::parse_comp_expected_var(l, c, &format!("{:?}", other)));
                }
            }
        }
    
        // 'in'
        if !self.match_tok(&Token::In) {
            let (l, c) = self.loc();
            return Err(msg::parse_comp_expected_in(l, c, &format!("{:?}", self.peek())));
        }
    
        let iter = self.parse_pipe()?;
    
        // Опциональный 'if'
        let condition = if self.match_tok(&Token::If) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
    
        self.skip_newlines();
        self.expect(&Token::RBrace, &msg::parse_expected_rbrace())?;
    
        Ok(Expr::StructComprehension {
            key: Box::new(key),
            value: Box::new(value),
            vars,
            iter: Box::new(iter),
            condition,
        })
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
        } else { None };
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

fn decompose_assign_target(expr: Expr) -> Option<(Box<Expr>, AssignKey)> {
    match expr {
        Expr::Field { target, name } => Some((target, AssignKey::Field(name))),
        Expr::Index { target, index } => Some((target, AssignKey::Index(index))),
        _ => None,
    }
}

fn expr_to_struct_key(e: &Expr) -> Option<String> {
    match e {
        Expr::Var(name) => Some(name.clone()),
        Expr::Str(s) => Some(s.clone()),
        Expr::Number(n) if n.fract() == 0.0 && n.is_finite() => {
            Some(format!("{}", *n as i64))
        }
        _ => None,
    }
}