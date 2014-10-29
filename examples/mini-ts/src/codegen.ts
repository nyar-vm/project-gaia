import {
    BinaryExpression,
    CallExpression,
    Expression,
    ExpressionStatement,
    FunctionDeclaration,
    Identifier,
    IfStatement,
    Literal,
    literalToGaiaConstant,
    Program,
    ReturnStatement,
    Statement,
    typeToGaiaType,
    UnaryExpression,
    VariableDeclaration,
    WhileStatement
} from './ast';

// 假设我们有一个gaia-frontend-wasm32库可以导入
// 在实际实现中，这里会导入实际的Gaia类型和函数
// import { GaiaInstruction, GaiaProgram, GaiaFunction, GaiaType } from 'gaia-frontend-wasm32';

// 为了示例，我们先定义一些基本的Gaia类型
class GaiaInstruction {
    constructor(public opcode: string, public operands: any[] = []) {
    }
}

class GaiaFunction {
    constructor(
        public name: string,
        public parameters: { name: string; type: string }[],
        public returnType: string,
        public instructions: GaiaInstruction[]
    ) {
    }
}

class GaiaProgram {
    constructor(public functions: GaiaFunction[]) {
    }
}

export class CodeGenerator {
    private program: GaiaProgram;
    private currentFunction?: GaiaFunction;
    private localVariables: Map<string, number> = new Map();
    private localIndex: number = 0;

    constructor() {
        this.program = new GaiaProgram([]);
    }

    public generate(program: Program): GaiaProgram {
        this.program = new GaiaProgram([]);

        for (const func of program.functions) {
            this.generateFunction(func);
        }

        return this.program;
    }

    private generateFunction(func: FunctionDeclaration): void {
        // 重置局部变量状态
        this.localVariables = new Map();
        this.localIndex = 0;

        // 创建参数映射
        for (let i = 0; i < func.parameters.length; i++) {
            this.localVariables.set(func.parameters[i].name, i);
        }

        // 生成函数体
        const instructions: GaiaInstruction[] = [];

        for (const stmt of func.body) {
            instructions.push(...this.generateStatement(stmt));
        }

        // 如果函数没有返回语句且返回类型不是void，添加默认返回
        if (func.returnType !== 'void' &&
            (!func.body.length || func.body[func.body.length - 1].kind !== 'ReturnStatement')) {
            instructions.push(new GaiaInstruction('Push', [literalToGaiaConstant({
                kind: 'Literal',
                type: func.returnType,
                value: func.returnType === 'boolean' ? false : 0
            })]));
            instructions.push(new GaiaInstruction('Return'));
        }

        // 创建Gaia函数
        const gaiaFunc = new GaiaFunction(
            func.name,
            func.parameters.map(p => ({name: p.name, type: typeToGaiaType(p.type)})),
            typeToGaiaType(func.returnType),
            instructions
        );

        this.program.functions.push(gaiaFunc);
    }

    private generateStatement(stmt: Statement): GaiaInstruction[] {
        switch (stmt.kind) {
            case 'VariableDeclaration':
                return this.generateVariableDeclaration(stmt);
            case 'ReturnStatement':
                return this.generateReturnStatement(stmt);
            case 'IfStatement':
                return this.generateIfStatement(stmt);
            case 'WhileStatement':
                return this.generateWhileStatement(stmt);
            case 'ExpressionStatement':
                return this.generateExpressionStatement(stmt);
            default:
                throw new Error(`Unsupported statement kind: ${stmt.kind}`);
        }
    }

    private generateVariableDeclaration(stmt: VariableDeclaration): GaiaInstruction[] {
        const instructions: GaiaInstruction[] = [];

        if (stmt.initializer) {
            instructions.push(...this.generateExpression(stmt.initializer));
        }

        // 分配局部变量索引
        const index = this.localIndex++;
        this.localVariables.set(stmt.name, index);

        // 如果有初始值，存储到局部变量
        if (stmt.initializer) {
            instructions.push(new GaiaInstruction('StoreLocal', [index]));
        }

        return instructions;
    }

    private generateReturnStatement(stmt: ReturnStatement): GaiaInstruction[] {
        const instructions: GaiaInstruction[] = [];

        if (stmt.value) {
            instructions.push(...this.generateExpression(stmt.value));
        }

        instructions.push(new GaiaInstruction('Return'));

        return instructions;
    }

    private generateIfStatement(stmt: IfStatement): GaiaInstruction[] {
        const instructions: GaiaInstruction[] = [];

        // 生成条件表达式
        instructions.push(...this.generateExpression(stmt.condition));

        // 创建跳转标签
        const elseLabel = `else_${Date.now()}`;
        const endLabel = `end_${Date.now()}`;

        // 条件跳转到else分支
        instructions.push(new GaiaInstruction('JumpIfFalse', [elseLabel]));

        // 生成then分支
        for (const thenStmt of stmt.thenBranch) {
            instructions.push(...this.generateStatement(thenStmt));
        }

        // 跳转到结束
        instructions.push(new GaiaInstruction('Jump', [endLabel]));

        // 生成else分支
        instructions.push(new GaiaInstruction('Label', [elseLabel]));

        if (stmt.elseBranch) {
            for (const elseStmt of stmt.elseBranch) {
                instructions.push(...this.generateStatement(elseStmt));
            }
        }

        // 结束标签
        instructions.push(new GaiaInstruction('Label', [endLabel]));

        return instructions;
    }

    private generateWhileStatement(stmt: WhileStatement): GaiaInstruction[] {
        const instructions: GaiaInstruction[] = [];

        // 创建循环标签
        const startLabel = `loop_start_${Date.now()}`;
        const endLabel = `loop_end_${Date.now()}`;

        // 循环开始标签
        instructions.push(new GaiaInstruction('Label', [startLabel]));

        // 生成条件表达式
        instructions.push(...this.generateExpression(stmt.condition));

        // 条件跳转到结束
        instructions.push(new GaiaInstruction('JumpIfFalse', [endLabel]));

        // 生成循环体
        for (const bodyStmt of stmt.body) {
            instructions.push(...this.generateStatement(bodyStmt));
        }

        // 跳回循环开始
        instructions.push(new GaiaInstruction('Jump', [startLabel]));

        // 循环结束标签
        instructions.push(new GaiaInstruction('Label', [endLabel]));

        return instructions;
    }

    private generateExpressionStatement(stmt: ExpressionStatement): GaiaInstruction[] {
        const instructions = this.generateExpression(stmt.expression);

        // 如果表达式有结果但未被使用，弹出它
        if (stmt.expression.kind !== 'CallExpression' &&
            stmt.expression.kind !== 'Assignment') {
            instructions.push(new GaiaInstruction('Pop'));
        }

        return instructions;
    }

    private generateExpression(expr: Expression): GaiaInstruction[] {
        switch (expr.kind) {
            case 'Literal':
                return this.generateLiteral(expr);
            case 'Identifier':
                return this.generateIdentifier(expr);
            case 'BinaryExpression':
                return this.generateBinaryExpression(expr);
            case 'UnaryExpression':
                return this.generateUnaryExpression(expr);
            case 'CallExpression':
                return this.generateCallExpression(expr);
            default:
                throw new Error(`Unsupported expression kind: ${expr.kind}`);
        }
    }

    private generateLiteral(expr: Literal): GaiaInstruction[] {
        return [new GaiaInstruction('Push', [literalToGaiaConstant(expr)])];
    }

    private generateIdentifier(expr: Identifier): GaiaInstruction[] {
        const index = this.localVariables.get(expr.name);

        if (index === undefined) {
            throw new Error(`Undefined variable: ${expr.name}`);
        }

        return [new GaiaInstruction('LoadLocal', [index])];
    }

    private generateBinaryExpression(expr: BinaryExpression): GaiaInstruction[] {
        const instructions: GaiaInstruction[] = [];

        // 生成左右操作数
        instructions.push(...this.generateExpression(expr.left));
        instructions.push(...this.generateExpression(expr.right));

        // 根据运算符生成对应的指令
        switch (expr.operator) {
            case 'Add':
                instructions.push(new GaiaInstruction('Add'));
                break;
            case 'Subtract':
                instructions.push(new GaiaInstruction('Subtract'));
                break;
            case 'Multiply':
                instructions.push(new GaiaInstruction('Multiply'));
                break;
            case 'Divide':
                instructions.push(new GaiaInstruction('Divide'));
                break;
            case 'Equal':
                instructions.push(new GaiaInstruction('CompareEqual'));
                break;
            case 'NotEqual':
                instructions.push(new GaiaInstruction('CompareEqual'));
                instructions.push(new GaiaInstruction('Not'));
                break;
            case 'LessThan':
                instructions.push(new GaiaInstruction('CompareLess'));
                break;
            case 'LessThanOrEqual':
                instructions.push(new GaiaInstruction('CompareLessOrEqual'));
                break;
            case 'GreaterThan':
                instructions.push(new GaiaInstruction('CompareGreater'));
                break;
            case 'GreaterThanOrEqual':
                instructions.push(new GaiaInstruction('CompareGreaterOrEqual'));
                break;
            default:
                throw new Error(`Unsupported binary operator: ${expr.operator}`);
        }

        return instructions;
    }

    private generateUnaryExpression(expr: UnaryExpression): GaiaInstruction[] {
        const instructions: GaiaInstruction[] = [];

        // 生成操作数
        instructions.push(...this.generateExpression(expr.operand));

        // 根据运算符生成对应的指令
        switch (expr.operator) {
            case 'Negate':
                instructions.push(new GaiaInstruction('Negate'));
                break;
            case 'Not':
                instructions.push(new GaiaInstruction('Not'));
                break;
            default:
                throw new Error(`Unsupported unary operator: ${expr.operator}`);
        }

        return instructions;
    }

    private generateCallExpression(expr: CallExpression): GaiaInstruction[] {
        const instructions: GaiaInstruction[] = [];

        // 生成参数
        for (const arg of expr.arguments) {
            instructions.push(...this.generateExpression(arg));
        }

        // 生成函数调用
        instructions.push(new GaiaInstruction('Call', [expr.callee.name]));

        return instructions;
    }
}