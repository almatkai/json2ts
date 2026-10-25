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
var ts = require("typescript");
var parser_1 = require("./parser");
exports.parse = parser_1.parse;
var printer_1 = require("./printer");
exports.print = printer_1.print;
var transformer_1 = require("./transformer");
exports.transform = transformer_1.transform;
var collapse_interfaces_1 = require("./collapse-interfaces");
exports.defaults = {
    prefix: "I",
    rootName: "RootObject"
};
function json2ts(validJsonString, options) {
    if (options === void 0) { options = {}; }
    var mergedOptions = __assign({}, exports.defaults, options);
    var _a = parser_1.parse(validJsonString, mergedOptions), stack = _a.stack, inputKind = _a.inputKind;
    switch (inputKind) {
        case ts.SyntaxKind.ArrayLiteralExpression:
        case ts.SyntaxKind.ObjectLiteralExpression: {
            var transformed = transformer_1.transform(stack, mergedOptions);
            var flattened = collapse_interfaces_1.collapseInterfaces(transformed);
            var printed = printer_1.print(flattened, inputKind, mergedOptions);
            return printed;
        }
        default: {
            var printed = printer_1.printLiteral(stack[0], inputKind, mergedOptions);
            return printed;
        }
    }
}
exports.json2ts = json2ts;
function json2tsMulti(validJsonStrings, options) {
    if (options === void 0) { options = {}; }
    var inputKinds = new Set([]);
    var mergedOptions = __assign({}, exports.defaults, options);
    var joined = validJsonStrings.reduce(function (all, json) {
        var _a = parser_1.parse(json, mergedOptions), stack = _a.stack, inputKind = _a.inputKind;
        inputKinds.add(inputKind);
        var transformed = transformer_1.transform(stack, mergedOptions);
        return all.concat(transformed);
    }, []);
    if (inputKinds.size > 1) {
        // todo handle mixed types
    }
    var flattened = collapse_interfaces_1.collapseInterfaces(joined);
    var printed = printer_1.print(flattened, Array.from(inputKinds)[0], mergedOptions);
    return printed;
}
exports.json2tsMulti = json2tsMulti;
if ((typeof window !== 'undefined') && ((typeof window.json2ts) === 'undefined')) {
    window.json2ts = json2ts;
    window.json2ts.parse = parser_1.parse;
    window.json2ts.transform = transformer_1.transform;
    window.json2ts.print = printer_1.print;
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaW5kZXguanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi9zcmMvaW5kZXgudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7OztBQUFBLCtCQUFpQztBQUNqQyxtQ0FBK0I7QUE2RDNCLGdCQTdESSxjQUFLLENBNkRKO0FBNURULHFDQUE4QztBQTZEMUMsZ0JBN0RJLGVBQUssQ0E2REo7QUE1RFQsNkNBQXdDO0FBNkRwQyxvQkE3REksdUJBQVMsQ0E2REo7QUE1RGIsNkRBQXlEO0FBUzVDLFFBQUEsUUFBUSxHQUFHO0lBQ3BCLE1BQU0sRUFBRSxHQUFHO0lBQ1gsUUFBUSxFQUFFLFlBQVk7Q0FDekIsQ0FBQztBQUVGLGlCQUF3QixlQUF1QixFQUFFLE9BQTJCO0lBQTNCLHdCQUFBLEVBQUEsWUFBMkI7SUFDeEUsSUFBTSxhQUFhLGdCQUNaLGdCQUFRLEVBQ1IsT0FBTyxDQUNiLENBQUM7SUFDSSxJQUFBLG1EQUEwRCxFQUF6RCxnQkFBSyxFQUFFLHdCQUFTLENBQTBDO0lBQ2pFLE1BQU0sQ0FBQyxDQUFDLFNBQVMsQ0FBQyxDQUFDLENBQUM7UUFDaEIsS0FBSyxFQUFFLENBQUMsVUFBVSxDQUFDLHNCQUFzQixDQUFDO1FBQzFDLEtBQUssRUFBRSxDQUFDLFVBQVUsQ0FBQyx1QkFBdUIsRUFBRSxDQUFDO1lBQ3pDLElBQU0sV0FBVyxHQUFHLHVCQUFTLENBQUMsS0FBSyxFQUFFLGFBQWEsQ0FBQyxDQUFDO1lBQ3BELElBQU0sU0FBUyxHQUFHLHdDQUFrQixDQUFDLFdBQVcsQ0FBQyxDQUFDO1lBQ2xELElBQU0sT0FBTyxHQUFHLGVBQUssQ0FBQyxTQUFTLEVBQUUsU0FBUyxFQUFFLGFBQWEsQ0FBQyxDQUFDO1lBQzNELE1BQU0sQ0FBQyxPQUFPLENBQUM7UUFDbkIsQ0FBQztRQUNELFNBQVMsQ0FBQztZQUNOLElBQU0sT0FBTyxHQUFHLHNCQUFZLENBQUMsS0FBSyxDQUFDLENBQUMsQ0FBQyxFQUFFLFNBQVMsRUFBRSxhQUFhLENBQUMsQ0FBQztZQUNqRSxNQUFNLENBQUMsT0FBTyxDQUFDO1FBQ25CLENBQUM7SUFDTCxDQUFDO0FBQ0wsQ0FBQztBQW5CRCwwQkFtQkM7QUFFRCxzQkFBNkIsZ0JBQTBCLEVBQUUsT0FBMkI7SUFBM0Isd0JBQUEsRUFBQSxZQUEyQjtJQUNoRixJQUFNLFVBQVUsR0FBRyxJQUFJLEdBQUcsQ0FBQyxFQUFFLENBQUMsQ0FBQztJQUMvQixJQUFNLGFBQWEsZ0JBQ1osZ0JBQVEsRUFDUixPQUFPLENBQ2IsQ0FBQztJQUNGLElBQU0sTUFBTSxHQUFHLGdCQUFnQixDQUFDLE1BQU0sQ0FBQyxVQUFDLEdBQUcsRUFBRSxJQUFJO1FBQ3ZDLElBQUEsd0NBQStDLEVBQTlDLGdCQUFLLEVBQUUsd0JBQVMsQ0FBK0I7UUFDdEQsVUFBVSxDQUFDLEdBQUcsQ0FBQyxTQUFTLENBQUMsQ0FBQztRQUMxQixJQUFNLFdBQVcsR0FBRyx1QkFBUyxDQUFDLEtBQUssRUFBRSxhQUFhLENBQUMsQ0FBQztRQUNwRCxNQUFNLENBQUMsR0FBRyxDQUFDLE1BQU0sQ0FBQyxXQUFXLENBQUMsQ0FBQztJQUNuQyxDQUFDLEVBQUUsRUFBRSxDQUFDLENBQUM7SUFFUCxFQUFFLENBQUMsQ0FBQyxVQUFVLENBQUMsSUFBSSxHQUFHLENBQUMsQ0FBQyxDQUFDLENBQUM7UUFDdEIsMEJBQTBCO0lBQzlCLENBQUM7SUFFRCxJQUFNLFNBQVMsR0FBRyx3Q0FBa0IsQ0FBQyxNQUFNLENBQUMsQ0FBQztJQUM3QyxJQUFNLE9BQU8sR0FBRyxlQUFLLENBQUMsU0FBUyxFQUFFLEtBQUssQ0FBQyxJQUFJLENBQUMsVUFBVSxDQUFDLENBQUMsQ0FBQyxDQUFDLEVBQUUsYUFBYSxDQUFDLENBQUM7SUFDM0UsTUFBTSxDQUFDLE9BQU8sQ0FBQztBQUNuQixDQUFDO0FBcEJELG9DQW9CQztBQVNELEVBQUUsQ0FBQyxDQUFDLENBQUMsT0FBTyxNQUFNLEtBQUssV0FBVyxDQUFDLElBQUksQ0FBQyxDQUFDLE9BQU8sTUFBTSxDQUFDLE9BQU8sQ0FBQyxLQUFLLFdBQVcsQ0FBQyxDQUFDLENBQUMsQ0FBQztJQUMvRSxNQUFNLENBQUMsT0FBTyxHQUFHLE9BQU8sQ0FBQztJQUN6QixNQUFNLENBQUMsT0FBTyxDQUFDLEtBQUssR0FBRyxjQUFLLENBQUM7SUFDN0IsTUFBTSxDQUFDLE9BQU8sQ0FBQyxTQUFTLEdBQUcsdUJBQVMsQ0FBQztJQUNyQyxNQUFNLENBQUMsT0FBTyxDQUFDLEtBQUssR0FBRyxlQUFLLENBQUM7QUFDakMsQ0FBQyIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCAqIGFzIHRzIGZyb20gJ3R5cGVzY3JpcHQnO1xuaW1wb3J0IHtwYXJzZX0gZnJvbSBcIi4vcGFyc2VyXCI7XG5pbXBvcnQge3ByaW50LCBwcmludExpdGVyYWx9IGZyb20gXCIuL3ByaW50ZXJcIjtcbmltcG9ydCB7dHJhbnNmb3JtfSBmcm9tIFwiLi90cmFuc2Zvcm1lclwiO1xuaW1wb3J0IHtjb2xsYXBzZUludGVyZmFjZXN9IGZyb20gXCIuL2NvbGxhcHNlLWludGVyZmFjZXNcIjtcblxuZXhwb3J0IGludGVyZmFjZSBKc29uVHNPcHRpb25zIHtcbiAgICBuYW1lc3BhY2U/OiBzdHJpbmdcbiAgICBmbG93PzogYm9vbGVhblxuICAgIHByZWZpeD86IHN0cmluZ1xuICAgIHJvb3ROYW1lPzogc3RyaW5nXG59XG5cbmV4cG9ydCBjb25zdCBkZWZhdWx0cyA9IHtcbiAgICBwcmVmaXg6IFwiSVwiLFxuICAgIHJvb3ROYW1lOiBcIlJvb3RPYmplY3RcIlxufTtcblxuZXhwb3J0IGZ1bmN0aW9uIGpzb24ydHModmFsaWRKc29uU3RyaW5nOiBzdHJpbmcsIG9wdGlvbnM6IEpzb25Uc09wdGlvbnMgPSB7fSk6IHN0cmluZyB7XG4gICAgY29uc3QgbWVyZ2VkT3B0aW9ucyA9IHtcbiAgICAgICAgLi4uZGVmYXVsdHMsXG4gICAgICAgIC4uLm9wdGlvbnNcbiAgICB9O1xuICAgIGNvbnN0IHtzdGFjaywgaW5wdXRLaW5kfSA9IHBhcnNlKHZhbGlkSnNvblN0cmluZywgbWVyZ2VkT3B0aW9ucyk7XG4gICAgc3dpdGNoIChpbnB1dEtpbmQpIHtcbiAgICAgICAgY2FzZSB0cy5TeW50YXhLaW5kLkFycmF5TGl0ZXJhbEV4cHJlc3Npb246XG4gICAgICAgIGNhc2UgdHMuU3ludGF4S2luZC5PYmplY3RMaXRlcmFsRXhwcmVzc2lvbjoge1xuICAgICAgICAgICAgY29uc3QgdHJhbnNmb3JtZWQgPSB0cmFuc2Zvcm0oc3RhY2ssIG1lcmdlZE9wdGlvbnMpO1xuICAgICAgICAgICAgY29uc3QgZmxhdHRlbmVkID0gY29sbGFwc2VJbnRlcmZhY2VzKHRyYW5zZm9ybWVkKTtcbiAgICAgICAgICAgIGNvbnN0IHByaW50ZWQgPSBwcmludChmbGF0dGVuZWQsIGlucHV0S2luZCwgbWVyZ2VkT3B0aW9ucyk7XG4gICAgICAgICAgICByZXR1cm4gcHJpbnRlZDtcbiAgICAgICAgfVxuICAgICAgICBkZWZhdWx0OiB7XG4gICAgICAgICAgICBjb25zdCBwcmludGVkID0gcHJpbnRMaXRlcmFsKHN0YWNrWzBdLCBpbnB1dEtpbmQsIG1lcmdlZE9wdGlvbnMpO1xuICAgICAgICAgICAgcmV0dXJuIHByaW50ZWQ7XG4gICAgICAgIH1cbiAgICB9XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBqc29uMnRzTXVsdGkodmFsaWRKc29uU3RyaW5nczogc3RyaW5nW10sIG9wdGlvbnM6IEpzb25Uc09wdGlvbnMgPSB7fSk6IHN0cmluZyB7XG4gICAgY29uc3QgaW5wdXRLaW5kcyA9IG5ldyBTZXQoW10pO1xuICAgIGNvbnN0IG1lcmdlZE9wdGlvbnMgPSB7XG4gICAgICAgIC4uLmRlZmF1bHRzLFxuICAgICAgICAuLi5vcHRpb25zXG4gICAgfTtcbiAgICBjb25zdCBqb2luZWQgPSB2YWxpZEpzb25TdHJpbmdzLnJlZHVjZSgoYWxsLCBqc29uKSA9PiB7XG4gICAgICAgIGNvbnN0IHtzdGFjaywgaW5wdXRLaW5kfSA9IHBhcnNlKGpzb24sIG1lcmdlZE9wdGlvbnMpO1xuICAgICAgICBpbnB1dEtpbmRzLmFkZChpbnB1dEtpbmQpO1xuICAgICAgICBjb25zdCB0cmFuc2Zvcm1lZCA9IHRyYW5zZm9ybShzdGFjaywgbWVyZ2VkT3B0aW9ucyk7XG4gICAgICAgIHJldHVybiBhbGwuY29uY2F0KHRyYW5zZm9ybWVkKTtcbiAgICB9LCBbXSk7XG5cbiAgICBpZiAoaW5wdXRLaW5kcy5zaXplID4gMSkge1xuICAgICAgICAvLyB0b2RvIGhhbmRsZSBtaXhlZCB0eXBlc1xuICAgIH1cblxuICAgIGNvbnN0IGZsYXR0ZW5lZCA9IGNvbGxhcHNlSW50ZXJmYWNlcyhqb2luZWQpO1xuICAgIGNvbnN0IHByaW50ZWQgPSBwcmludChmbGF0dGVuZWQsIEFycmF5LmZyb20oaW5wdXRLaW5kcylbMF0sIG1lcmdlZE9wdGlvbnMpO1xuICAgIHJldHVybiBwcmludGVkO1xufVxuXG5leHBvcnQge1xuICAgIHBhcnNlLFxuICAgIHByaW50LFxuICAgIHRyYW5zZm9ybVxufVxuXG5kZWNsYXJlIHZhciB3aW5kb3c7XG5pZiAoKHR5cGVvZiB3aW5kb3cgIT09ICd1bmRlZmluZWQnKSAmJiAoKHR5cGVvZiB3aW5kb3cuanNvbjJ0cykgPT09ICd1bmRlZmluZWQnKSkge1xuICAgIHdpbmRvdy5qc29uMnRzID0ganNvbjJ0cztcbiAgICB3aW5kb3cuanNvbjJ0cy5wYXJzZSA9IHBhcnNlO1xuICAgIHdpbmRvdy5qc29uMnRzLnRyYW5zZm9ybSA9IHRyYW5zZm9ybTtcbiAgICB3aW5kb3cuanNvbjJ0cy5wcmludCA9IHByaW50O1xufVxuIl19