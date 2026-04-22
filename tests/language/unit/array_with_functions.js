// Test: functions inside arrays (stored and length checked)
function test() {
    return [function(x) { return x + 1; }, function(x) { return x * 2; }, function(x) { return x - 1; }].length;
}
test()
