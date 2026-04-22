// ERROR: function using global variable not in its scope
function compute() {
    return missing_var + 1;
}
compute()
