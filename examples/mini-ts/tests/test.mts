function main(): number {
    let x: number = 10;
    let y: number = 20;
    let z: number = x + y;
    return z;
}

function add(a: number, b: number): number {
    return a + b;
}

function factorial(n: number): number {
    if (n <= 1) {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

function fibonacci(n: number): number {
    if (n <= 1) {
        return n;
    } else {
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}