// Nested destructuring simulation
function test() {
    function extract({name, config}) {
        return config;
    }
    return extract({name: "test", config: {debug: true}});
}
test()
