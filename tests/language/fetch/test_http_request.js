// http.request() — universal request with options
function test() {
    function handleResponse(res) {
        return res.ok;
    }
    return http.request("https://jsonplaceholder.typicode.com/posts/1", {
        method: "DELETE"
    }).then(handleResponse);
}
test()
