// http.post() — POST request with JSON body
function test() {
    function handleResponse(res) {
        return res.status;
    }
    return http.post("https://jsonplaceholder.typicode.com/posts", { title: "test", body: "hello", userId: 1 }).then(handleResponse);
}
test()
