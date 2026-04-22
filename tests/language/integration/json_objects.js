// JSON.parse + object destructuring
function test() {
    function extractName({name}) {
        return name;
    }
    return extractName(JSON.parse('{"name":"max","age":30}'));
}
test()
