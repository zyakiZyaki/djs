// Test: not matcher
function runTests() {
    describe("expect.not", function() {
        test("not equal", function() {
            expect(1).not.toEqual(2);
        });
        test("not truthy", function() {
            expect(0).not.toBeTruthy();
        });
        test("not falsy", function() {
            expect(1).not.toBeFalsy();
        });
    });
    return 0;
}
runTests()
