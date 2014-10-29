// AST节点类型定义
export type BinaryOperator =
    'Add'
    | 'Subtract'
    | 'Multiply'
    | 'Divide'
    | 'Equal'
    | 'NotEqual'
    | 'LessThan'
    | 'LessThanOrEqual'
    | 'GreaterThan'
    | 'GreaterThanOrEqual';
export type UnaryOperator = 'Negate' | 'Not';

export type Type = 'number' | 'string' | 'boolean' | 'void';

export interface Literal {
    kind: 'Literal';
    type: Type;
    value: number | string | boolean;
}

export interface Identifier {
    kind: 'Identifier';
    name: string;
}

export interface BinaryExpression {
    kind: 'BinaryExpression';
    left: Expression;
    operator: BinaryOperator;
    right: Expression;
}

export interface UnaryExpression {
    kind: 'UnaryExpression';
    operator: UnaryOperator;
    operand: Expression;
}

export interface CallExpression {
    kind: 'CallExpression';
    callee: Identifier;
    arguments: Expression[];
}

export type Expression = Literal | Identifier | BinaryExpression | UnaryExpression | CallExpression;

export interface VariableDeclaration {
    kind: 'VariableDeclaration';
    name: string;
    type: Type;
    initializer?: Expression;
}

export interface FunctionDeclaration {
    kind: 'FunctionDeclaration';
    name: string;
    parameters: Parameter[];
    returnType: Type;
    body: Statement[];
}

export interface ReturnStatement {
    kind: 'ReturnStatement';
    value?: Expression;
}

export interface IfStatement {
    kind: 'IfStatement';
    condition: Expression;
    thenBranch: Statement[];
    elseBranch?: Statement[];
}

export interface WhileStatement {
    kind: 'WhileStatement';
    condition: Expression;
    body: Statement[];
}

export interface ExpressionStatement {
    kind: 'ExpressionStatement';
    expression: Expression;
}

export type Statement =
    | VariableDeclaration
    | FunctionDeclaration
    | ReturnStatement
    | IfStatement
    | WhileStatement
    | ExpressionStatement;

export interface Parameter {
    name: string;
    type: Type;
}

export interface Program {
    functions: FunctionDeclaration[];
}

// 将TypeScript类型转换为Gaia类型
export function typeToGaiaType(type: Type): string {
    switch (type) {
        case 'number':
            return 'F64'; // TypeScript的number对应Gaia的F64
        case 'string':
            return 'String';
        case 'boolean':
            return 'Bool';
        case 'void':
            return 'Void';
        default:
            throw new Error(`Unknown type: ${type}`);
    }
}

// 将字面量转换为Gaia常量
export function literalToGaiaConstant(literal: Literal): any {
    switch (literal.type) {
        case 'number':
            return {kind: 'Float', value: literal.value as number};
        case 'boolean':
            return {kind: 'Boolean', value: literal.value as boolean};
        case 'string':
            return {kind: 'String', value: literal.value as string};
        default:
            throw new Error(`Cannot convert literal of type ${literal.type} to Gaia constant`);
    }
}