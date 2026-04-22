// Test: built-in test framework - expect/toEqual
function runTests() {
    describe("expect.toEqual", function() {
        test("numbers equal", function() {
            expect(42).toEqual(42);
        });
        test("strings equal", function() {
            expect("hello").toEqual("hello");
        });
        test("booleans equal", function() {
            expect(true).toEqual(true);
        });
    });
    return 0;
}
runTests()
