// ERROR: calling function from parent scope
function outer() {
    function helper() { return 1; }
    function inner() {
        function deep() { return helper(); }
        return deep();
    }
    return inner();
}
outer()
