// http.patch() with real sites
function test() {
    function handleResponse(res) {
        return res.status;
    }
    return http.patch("https://httpbin.org/patch", { title: "test", body: "hello" }).then(handleResponse);
}
test()