import {Lexer, Token} from './lexer';
import {
    BinaryExpression,
    BinaryOperator,
    CallExpression,
    Expression,
    ExpressionStatement,
    FunctionDeclaration,
    Identifier,
    IfStatement,
    Literal,
    Parameter,
    Program,
    ReturnStatement,
    Statement,
    Type,
    UnaryExpression,
    UnaryOperator,
    VariableDeclaration,
    WhileStatement
} from './ast';

export class Parser {
    private tokens: Token[];
    private current: number = 0;

    constructor(input: string) {
        const lexer = new Lexer(input);
        this.tokens = lexer.tokenize();
    }

    private peek(): Token {
        return this.tokens[this.current];
    }

    private previous(): Token {
        return this.tokens[this.current - 1];
    }

    private isAtEnd(): boolean {
        return this.peek().kind === 'EOF';
    }

    private advance(): Token {
        if (!this.isAtEnd()) this.current++;
        return this.previous();
    }

    private check(kind: string): boolean {
        if (this.isAtEnd()) return false;
        return this.peek().kind === kind;
    }

    private match(...kinds: string[]): boolean {
        for (const kind of kinds) {
            if (this.check(kind)) {
                this.advance();
                return true;
            }
        }
        return false;
    }

    private consume(kind: string, message: string): Token {
        if (this.check(kind)) return this.advance();

        throw new Error(`${message}. Got ${this.peek().kind} instead.`);
    }

    private error(message: string): never {
        throw new Error(message);
    }

    public parse(): Program {
        const functions: FunctionDeclaration[] = [];

        while (!this.isAtEnd()) {
            functions.push(this.functionDeclaration());
        }

        return {functions};
    }

    private functionDeclaration(): FunctionDeclaration {
        this.consume('Keyword', "Expected 'function' keyword");
        const name = this.consume('Identifier', "Expected function name").value;

        this.consume('Punctuation', "Expected '('");
        const parameters: Parameter[] = [];

        if (!this.check('Punctuation') || this.peek().value !== ')') {
            do {
                const paramName = this.consume('Identifier', "Expected parameter name").value;
                this.consume('Punctuation', "Expected ':'");
                const paramType = this.type();
                parameters.push({name: paramName, type: paramType});
            } while (this.match('Punctuation', ','));
        }

        this.consume('Punctuation', "Expected ')'");
        this.consume('Punctuation', "Expected ':'");
        const returnType = this.type();

        this.consume('Punctuation', "Expected '{'");
        const body: Statement[] = [];

        while (!this.check('Punctuation') || this.peek().value !== '}') {
            body.push(this.statement());
        }

        this.consume('Punctuation', "Expected '}'");

        return {
            kind: 'FunctionDeclaration',
            name,
            parameters,
            returnType,
            body
        };
    }

    private statement(): Statement {
        if (this.match('Keyword', 'let')) {
            return this.variableDeclaration();
        }

        if (this.match('Keyword', 'return')) {
            return this.returnStatement();
        }

        if (this.match('Keyword', 'if')) {
            return this.ifStatement();
        }

        if (this.match('Keyword', 'while')) {
            return this.whileStatement();
        }

        return this.expressionStatement();
    }

    private variableDeclaration(): VariableDeclaration {
        const name = this.consume('Identifier', "Expected variable name").value;

        let type: Type | undefined;
        let initializer: Expression | undefined;

        if (this.match('Punctuation', ':')) {
            type = this.type();
        }

        if (this.match('Operator', '=')) {
            initializer = this.expression();
        }

        this.consume('Punctuation', "Expected ';'");

        return {
            kind: 'VariableDeclaration',
            name,
            type: type || 'void',
            initializer
        };
    }

    private returnStatement(): ReturnStatement {
        let value: Expression | undefined;

        if (!this.check('Punctuation') || this.peek().value !== ';') {
            value = this.expression();
        }

        this.consume('Punctuation', "Expected ';'");

        return {
            kind: 'ReturnStatement',
            value
        };
    }

    private ifStatement(): IfStatement {
        this.consume('Punctuation', "Expected '('");
        const condition = this.expression();
        this.consume('Punctuation', "Expected ')'");

        this.consume('Punctuation', "Expected '{'");
        const thenBranch: Statement[] = [];

        while (!this.check('Punctuation') || this.peek().value !== '}') {
            thenBranch.push(this.statement());
        }

        this.consume('Punctuation', "Expected '}'");

        let elseBranch: Statement[] | undefined;

        if (this.match('Keyword', 'else')) {
            this.consume('Punctuation', "Expected '{'");
            elseBranch = [];

            while (!this.check('Punctuation') || this.peek().value !== '}') {
                elseBranch.push(this.statement());
            }

            this.consume('Punctuation', "Expected '}'");
        }

        return {
            kind: 'IfStatement',
            condition,
            thenBranch,
            elseBranch
        };
    }

    private whileStatement(): WhileStatement {
        this.consume('Punctuation', "Expected '('");
        const condition = this.expression();
        this.consume('Punctuation', "Expected ')'");

        this.consume('Punctuation', "Expected '{'");
        const body: Statement[] = [];

        while (!this.check('Punctuation') || this.peek().value !== '}') {
            body.push(this.statement());
        }

        this.consume('Punctuation', "Expected '}'");

        return {
            kind: 'WhileStatement',
            condition,
            body
        };
    }

    private expressionStatement(): ExpressionStatement {
        const expr = this.expression();
        this.consume('Punctuation', "Expected ';'");

        return {
            kind: 'ExpressionStatement',
            expression: expr
        };
    }

    private expression(): Expression {
        return this.assignment();
    }

    private assignment(): Expression {
        const expr = this.logicalOr();

        if (this.match('Operator', '=')) {
            const value = this.assignment();

            if (expr.kind === 'Identifier') {
                return {
                    kind: 'BinaryExpression',
                    left: expr,
                    operator: 'Equal',
                    right: value
                };
            }

            this.error("Invalid assignment target");
        }

        return expr;
    }

    private logicalOr(): Expression {
        let expr = this.logicalAnd();

        while (this.match('Operator', '||')) {
            const operator = 'Or' as BinaryOperator;
            const right = this.logicalAnd();
            expr = {
                kind: 'BinaryExpression',
                left: expr,
                operator,
                right
            };
        }

        return expr;
    }

    private logicalAnd(): Expression {
        let expr = this.equality();

        while (this.match('Operator', '&&')) {
            const operator = 'And' as BinaryOperator;
            const right = this.equality();
            expr = {
                kind: 'BinaryExpression',
                left: expr,
                operator,
                right
            };
        }

        return expr;
    }

    private equality(): Expression {
        let expr = this.comparison();

        while (this.match('Operator', '==', '!=')) {
            const operator = this.previous().value === '==' ? 'Equal' : 'NotEqual' as BinaryOperator;
            const right = this.comparison();
            expr = {
                kind: 'BinaryExpression',
                left: expr,
                operator,
                right
            };
        }

        return expr;
    }

    private comparison(): Expression {
        let expr = this.term();

        while (this.match('Operator', '<', '<=', '>', '>=')) {
            let operator: BinaryOperator;
            const value = this.previous().value;

            switch (value) {
                case '<':
                    operator = 'LessThan';
                    break;
                case '<=':
                    operator = 'LessThanOrEqual';
                    break;
                case '>':
                    operator = 'GreaterThan';
                    break;
                case '>=':
                    operator = 'GreaterThanOrEqual';
                    break;
                default:
                    operator = 'LessThan'; // 不应该发生
            }

            const right = this.term();
            expr = {
                kind: 'BinaryExpression',
                left: expr,
                operator,
                right
            };
        }

        return expr;
    }

    private term(): Expression {
        let expr = this.factor();

        while (this.match('Operator', '+', '-')) {
            const operator = this.previous().value === '+' ? 'Add' : 'Subtract' as BinaryOperator;
            const right = this.factor();
            expr = {
                kind: 'BinaryExpression',
                left: expr,
                operator,
                right
            };
        }

        return expr;
    }

    private factor(): Expression {
        let expr = this.unary();

        while (this.match('Operator', '*', '/')) {
            const operator = this.previous().value === '*' ? 'Multiply' : 'Divide' as BinaryOperator;
            const right = this.unary();
            expr = {
                kind: 'BinaryExpression',
                left: expr,
                operator,
                right
            };
        }

        return expr;
    }

    private unary(): Expression {
        if (this.match('Operator', '-', '!')) {
            const operator = this.previous().value === '-' ? 'Negate' : 'Not' as UnaryOperator;
            const right = this.unary();
            return {
                kind: 'UnaryExpression',
                operator,
                operand: right
            };
        }

        return this.call();
    }

    private call(): Expression {
        let expr = this.primary();

        while (true) {
            if (this.match('Punctuation', '(')) {
                expr = this.finishCall(expr);
            } else {
                break;
            }
        }

        return expr;
    }

    private finishCall(callee: Expression): Expression {
        const args: Expression[] = [];

        if (!this.check('Punctuation') || this.peek().value !== ')') {
            do {
                args.push(this.expression());
            } while (this.match('Punctuation', ','));
        }

        this.consume('Punctuation', "Expected ')'");

        if (callee.kind !== 'Identifier') {
            this.error("Can only call identifiers");
        }

        return {
            kind: 'CallExpression',
            callee: callee as Identifier,
            arguments: args
        };
    }

    private primary(): Expression {
        if (this.match('Number')) {
            const value = this.previous().value as number;
            return {
                kind: 'Literal',
                type: 'number', // TypeScript中所有数字都是number类型
                value
            };
        }

        if (this.match('String')) {
            return {
                kind: 'Literal',
                type: 'string',
                value: this.previous().value
            };
        }

        if (this.match('Boolean')) {
            return {
                kind: 'Literal',
                type: 'boolean',
                value: this.previous().value
            };
        }

        if (this.match('Identifier')) {
            return {
                kind: 'Identifier',
                name: this.previous().value
            };
        }

        if (this.match('Punctuation', '(')) {
            const expr = this.expression();
            this.consume('Punctuation', "Expected ')'");
            return expr;
        }

        throw new Error(`Expected expression, got ${this.peek().kind}`);
    }

    private type(): Type {
        const typeName = this.consume('Keyword', "Expected type name").value;

        switch (typeName) {
            case 'number':
                return 'number';
            case 'string':
                return 'string';
            case 'boolean':
                return 'boolean';
            case 'void':
                return 'void';
            default:
                throw new Error(`Unknown type: ${typeName}`);
        }
    }
}