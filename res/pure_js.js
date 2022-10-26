export function something() {
    var x = 10.1;
    return x;
}

export function main() {
    console.log("⭐ hello, VM! ⭐");
    var x = 10 * 5; // 50
    var y = 4.4 + x; // 54.4
    x = y - something(); // 54.4 - 10.1 = 44.3
    var z = sin(x - something()); // sin(44.3 - 10.1) = 0.35
}