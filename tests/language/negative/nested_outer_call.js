// ERROR: function calling function defined in another function (not nested, not param)
function outer() {
    function inner_a() { return 1; }
    function inner_b() { return inner_a(); }
    return inner_b();
}
outer()
