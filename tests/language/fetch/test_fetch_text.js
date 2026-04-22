// fetch() .text() method
function test() {
    function handleResponse(res) {
        return res.text();
    }
    return fetch("https://jsonplaceholder.typicode.com/posts/1").then(handleResponse);
}
test()
