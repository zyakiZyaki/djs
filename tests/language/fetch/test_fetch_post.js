// fetch() with POST and body
function test() {
    function handleResponse(res) {
        return res.status;
    }
    return fetch("https://jsonplaceholder.typicode.com/posts", {
        method: "POST",
        body: { title: "test", body: "hello", userId: 1 }
    }).then(handleResponse);
}
test()
