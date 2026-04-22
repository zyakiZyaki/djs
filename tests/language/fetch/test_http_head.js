// http.head() with real sites
function test() {
    function handleResponse(res) {
        return res.status;
    }
    return http.head("https://httpbin.org/get").then(handleResponse);
}
test()