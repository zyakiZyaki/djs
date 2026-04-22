// http.get() → GET /health → parse JSON
function test() {
    function handleResponse(res) {
        return res.json();
    }
    return http.get("http://localhost:8888/health").then(handleResponse);
}
test()
