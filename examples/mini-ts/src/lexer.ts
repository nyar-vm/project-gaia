// 词法分析器
export type Token =
    | { kind: 'Keyword'; value: string }
    | { kind: 'Identifier'; value: string }
    | { kind: 'Number'; value: number }
    | { kind: 'String'; value: string }
    | { kind: 'Boolean'; value: boolean }
    | { kind: 'Operator'; value: string }
    | { kind: 'Punctuation'; value: string }
    | { kind: 'EOF' };

export class Lexer {
    private input: string;
    private position: number = 0;
    private line: number = 1;
    private column: number = 1;

    constructor(input: string) {
        this.input = input;
    }

    private peek(offset: number = 0): string {
        const pos = this.position + offset;
        return pos >= this.input.length ? '\0' : this.input[pos];
    }

    private advance(): string {
        if (this.position >= this.input.length) return '\0';

        const char = this.input[this.position];
        this.position++;

        if (char === '\n') {
            this.line++;
            this.column = 1;
        } else {
            this.column++;
        }

        return char;
    }

    private skipWhitespace(): void {
        while (/\s/.test(this.peek()) && this.peek() !== '\0') {
            this.advance();
        }
    }

    private skipComment(): void {
        if (this.peek() === '/' && this.peek(1) === '/') {
            // 单行注释
            while (this.peek() !== '\n' && this.peek() !== '\0') {
                this.advance();
            }
        }
    }

    private readString(): Token {
        const quote = this.advance(); // 消耗开始引号
        let value = '';

        while (this.peek() !== quote && this.peek() !== '\0') {
            const char = this.advance();

            // 处理转义字符
            if (char === '\\' && this.peek() !== '\0') {
                const escaped = this.advance();
                switch (escaped) {
                    case 'n':
                        value += '\n';
                        break;
                    case 't':
                        value += '\t';
                        break;
                    case 'r':
                        value += '\r';
                        break;
                    case '\\':
                        value += '\\';
                        break;
                    case quote:
                        value += quote;
                        break;
                    default:
                        value += escaped;
                }
            } else {
                value += char;
            }
        }

        if (this.peek() === quote) {
            this.advance(); // 消耗结束引号
        } else {
            throw new Error(`Unterminated string at line ${this.line}, column ${this.column}`);
        }

        return {kind: 'String', value};
    }

    private readNumber(): Token {
        let value = '';
        let isFloat = false;

        while (/\d/.test(this.peek())) {
            value += this.advance();
        }

        if (this.peek() === '.' && /\d/.test(this.peek(1))) {
            isFloat = true;
            value += this.advance(); // 消耗小数点

            while (/\d/.test(this.peek())) {
                value += this.advance();
            }
        }

        return {
            kind: 'Number',
            value: isFloat ? parseFloat(value) : parseInt(value, 10)
        };
    }

    private readIdentifier(): Token {
        let value = '';

        while (/[\w_]/.test(this.peek())) {
            value += this.advance();
        }

        // 检查是否是关键字
        const keywords = [
            'let', 'const', 'fn', 'return', 'if', 'else', 'while',
            'true', 'false', 'i32', 'i64', 'f32', 'f64', 'bool', 'string', 'void'
        ];

        if (keywords.includes(value)) {
            if (value === 'true' || value === 'false') {
                return {kind: 'Boolean', value: value === 'true'};
            }
            return {kind: 'Keyword', value};
        }

        return {kind: 'Identifier', value};
    }

    public nextToken(): Token {
        this.skipWhitespace();

        // 处理注释
        if (this.peek() === '/' && this.peek(1) === '/') {
            this.skipComment();
            return this.nextToken();
        }

        const char = this.peek();

        if (char === '\0') {
            return {kind: 'EOF'};
        }

        // 字符串字面量
        if (char === '"' || char === "'") {
            return this.readString();
        }

        // 数字字面量
        if (/\d/.test(char)) {
            return this.readNumber();
        }

        // 标识符或关键字
        if (/[\w_]/.test(char)) {
            return this.readIdentifier();
        }

        // 运算符和标点
        this.advance();

        // 多字符运算符
        if (char === '=' && this.peek() === '=') {
            this.advance();
            return {kind: 'Operator', value: '=='};
        }

        if (char === '!' && this.peek() === '=') {
            this.advance();
            return {kind: 'Operator', value: '!='};
        }

        if (char === '<' && this.peek() === '=') {
            this.advance();
            return {kind: 'Operator', value: '<='};
        }

        if (char === '>' && this.peek() === '=') {
            this.advance();
            return {kind: 'Operator', value: '>='};
        }

        // 单字符运算符和标点
        if (['+', '-', '*', '/', '=', '!', '<', '>', '&', '|', '^', '~'].includes(char)) {
            return {kind: 'Operator', value: char};
        }

        if (['(', ')', '{', '}', '[', ']', ',', ';', ':'].includes(char)) {
            return {kind: 'Punctuation', value: char};
        }

        throw new Error(`Unexpected character '${char}' at line ${this.line}, column ${this.column}`);
    }

    public tokenize(): Token[] {
        const tokens: Token[] = [];
        let token: Token;

        do {
            token = this.nextToken();
            tokens.push(token);
        } while (token.kind !== 'EOF');

        return tokens;
    }
}