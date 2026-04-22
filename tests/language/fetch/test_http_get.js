// http.get() — GET request
function test() {
    function handleResponse(res) {
        return res.status;
    }
    return http.get("https://jsonplaceholder.typicode.com/posts/1").then(handleResponse);
}
test()
