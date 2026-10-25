"use strict";
var __assign = (this && this.__assign) || Object.assign || function(t) {
    for (var s, i = 1, n = arguments.length; i < n; i++) {
        s = arguments[i];
        for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p))
            t[p] = s[p];
    }
    return t;
};
Object.defineProperty(exports, "__esModule", { value: true });
var parser_1 = require("./parser");
exports.parse = parser_1.parse;
var printer_1 = require("./printer");
exports.print = printer_1.print;
var transformer_1 = require("./transformer");
exports.transform = transformer_1.transform;
exports.defaults = {
    prefix: "I",
    rootName: "RootObject"
};
function json2ts(validJsonString, options) {
    if (options === void 0) { options = {}; }
    var merged = __assign({}, exports.defaults, options);
    var parsed = parser_1.parse(validJsonString, merged);
    var transformed = transformer_1.transform(parsed, merged);
    var printed = printer_1.print(transformed, merged);
    return printed;
}
exports.json2ts = json2ts;
if ((typeof window !== 'undefined') && ((typeof window.json2ts) === 'undefined')) {
    window.json2ts = json2ts;
    window.json2ts.parse = parser_1.parse;
    window.json2ts.transform = transformer_1.transform;
    window.json2ts.print = printer_1.print;
}
