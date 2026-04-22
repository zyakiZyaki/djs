// fetch() basic test
function test() {
    function handleResponse(res) {
        return res.json();
    }
    function handleError(err) {
        return { error: "failed" };
    }
    return fetch("https://jsonplaceholder.typicode.com/posts/1").then(handleResponse);
}
test()
