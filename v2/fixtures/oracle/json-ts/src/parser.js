"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var ts = require("typescript");
var needsQuotes = require("needsquotes");
function walk(sourceFile) {
    var stack = [];
    var elementStack = [];
    function push(element) {
        var parent = elementStack[elementStack.length - 1];
        var siblings = (parent && parent.body) ? parent.body : stack;
        siblings.push(element);
    }
    eachProp(sourceFile);
    return stack;
    function addFromArrayElement(incoming) {
        switch (incoming.kind) {
            case ts.SyntaxKind.NullKeyword:
            case ts.SyntaxKind.TrueKeyword:
            case ts.SyntaxKind.FalseKeyword:
            case ts.SyntaxKind.NumericLiteral:
            case ts.SyntaxKind.StringLiteral: {
                push(literalTypeFromArrayElement(incoming, incoming.kind));
                break;
            }
            case ts.SyntaxKind.PrefixUnaryExpression: {
                push(literalTypeFromArrayElement(incoming, ts.SyntaxKind.NumericLiteral));
                break;
            }
            case ts.SyntaxKind.ObjectLiteralExpression: {
                var elem = {
                    kind: ts.SyntaxKind.ObjectLiteralExpression,
                    _kind: "ObjectLiteralExpression",
                    interfaceCandidate: true,
                    body: [],
                };
                push(elem);
                elementStack.push(elem);
                eachProp(incoming.properties);
                elementStack.pop();
                break;
            }
            case ts.SyntaxKind.ArrayLiteralExpression: {
                var elem = {
                    kind: ts.SyntaxKind.ArrayLiteralExpression,
                    _kind: "ArrayLiteralExpression",
                    body: [],
                };
                push(elem);
                elementStack.push(elem);
                eachProp(incoming.elements);
                elementStack.pop();
                break;
            }
        }
    }
    function eachProp(properties) {
        properties.forEach(function (prop) {
            if (!prop.initializer) {
                return addFromArrayElement(prop);
            }
            else {
                switch (prop.initializer.kind) {
                    case ts.SyntaxKind.TrueKeyword:
                    case ts.SyntaxKind.FalseKeyword:
                    case ts.SyntaxKind.NullKeyword:
                    case ts.SyntaxKind.StringLiteral:
                    case ts.SyntaxKind.NumericLiteral: {
                        push(literalTypeFromProp(prop, prop.initializer.kind));
                        break;
                    }
                    case ts.SyntaxKind.PrefixUnaryExpression: {
                        push(literalTypeFromProp(prop, ts.SyntaxKind.NumericLiteral));
                        break;
                    }
                    case ts.SyntaxKind.ObjectLiteralExpression: {
                        var quotes = needsQuotes(prop.name.text).needsQuotes;
                        var interfaceCandidate = (quotes === false);
                        var elem = {
                            name: prop.name.text,
                            body: [],
                            interfaceCandidate: interfaceCandidate,
                            kind: ts.SyntaxKind.ObjectLiteralExpression,
                            _kind: "ObjectLiteralExpression"
                        };
                        push(elem);
                        elementStack.push(elem);
                        eachProp(prop.initializer.properties);
                        elementStack.pop();
                        break;
                    }
                    case ts.SyntaxKind.ArrayLiteralExpression: {
                        var elem = {
                            name: prop.name.text,
                            body: [],
                            kind: ts.SyntaxKind.ArrayLiteralExpression,
                            _kind: "ArrayLiteralExpression"
                        };
                        push(elem);
                        elementStack.push(elem);
                        eachProp(prop.initializer.elements);
                        elementStack.pop();
                        break;
                    }
                }
            }
        });
    }
    function literalTypeFromProp(prop, kind) {
        return {
            name: prop.name.text,
            value: prop.initializer.text,
            kind: kind,
        };
    }
    function literalTypeFromArrayElement(element, kind) {
        return {
            kind: kind,
            name: element.text,
            value: element.text,
        };
    }
}
function parse(string, options) {
    var input = "const ROOTOBJ = " + string;
    var sourceFile = ts.createSourceFile('json.ts', input, ts.ScriptTarget.ES2015, /*setParentNodes */ true);
    // delint it
    var _json = sourceFile.statements[0];
    // console.log(sourceFile.statements[0].declarationList.declarations[0].initializer.properties);
    // elementStack.push({name: 'root', body: []});
    var stack = walk(_json.declarationList.declarations[0].initializer.properties);
    return stack;
}
exports.parse = parse;
