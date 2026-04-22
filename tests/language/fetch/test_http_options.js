// http.options() with real sites
function test() {
    function handleResponse(res) {
        return res.status;
    }
    return http.options("https://httpbin.org/get").then(handleResponse);
}
test()