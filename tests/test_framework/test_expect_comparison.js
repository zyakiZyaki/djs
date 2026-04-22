// Test: comparison matchers
function runTests() {
    describe("expect.comparisons", function() {
        test("toBeGreaterThan", function() {
            expect(5).toBeGreaterThan(3);
        });
        test("toBeLessThan", function() {
            expect(3).toBeLessThan(5);
        });
        test("toBeTruthy", function() {
            expect(1).toBeTruthy();
        });
        test("toBeFalsy", function() {
            expect(0).toBeFalsy();
        });
    });
    return 0;
}
runTests()
