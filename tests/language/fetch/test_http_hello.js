// http.get() → GET /hello
function test() {
    function handleResponse(res) {
        return res.text();
    }
    return http.get("http://localhost:8888/hello").then(handleResponse);
}
test()
