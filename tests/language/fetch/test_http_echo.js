// http.post() → POST /echo
function test() {
    function handleResponse(res) {
        return res.json();
    }
    return http.post("http://localhost:8888/echo", { name: "max", age: 30 }).then(handleResponse);
}
test()
