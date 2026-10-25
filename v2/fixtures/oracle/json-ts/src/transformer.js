"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var ts = require("typescript");
var immutable_1 = require("immutable");
var needsQuotes = require("needsquotes");
var collapse_interfaces_1 = require("./collapse-interfaces");
var _a = require('../_'), startCase = _a.startCase, toLower = _a.toLower;
exports.log = function (input) { return console.log('--\n', JSON.stringify(input, null, 2)); };
var kindMap = (_b = {},
    _b[ts.SyntaxKind.NullKeyword] = ts.SyntaxKind.NullKeyword,
    _b[ts.SyntaxKind.StringLiteral] = ts.SyntaxKind.StringKeyword,
    _b[ts.SyntaxKind.FirstLiteralToken] = ts.SyntaxKind.NumberKeyword,
    _b[ts.SyntaxKind.TrueKeyword] = ts.SyntaxKind.BooleanKeyword,
    _b[ts.SyntaxKind.FalseKeyword] = ts.SyntaxKind.BooleanKeyword,
    _b[ts.SyntaxKind.NumericLiteral] = ts.SyntaxKind.NumberKeyword,
    _b);
function namedProp(member) {
    var qs = needsQuotes(member.name);
    var output = qs.needsQuotes ? qs.quotedValue : member.name;
    var prop = ts.createNode(ts.SyntaxKind.PropertySignature);
    prop.name = ts.createIdentifier(output);
    if (member.optional) {
        prop.questionToken = ts.createNode(ts.SyntaxKind.QuestionToken);
    }
    return prop;
}
exports.namedProp = namedProp;
var safeUnions = immutable_1.Set([
    ts.SyntaxKind.TrueKeyword,
    ts.SyntaxKind.FalseKeyword,
    ts.SyntaxKind.StringLiteral,
    ts.SyntaxKind.NumericLiteral,
    ts.SyntaxKind.PrefixUnaryExpression,
    ts.SyntaxKind.NullKeyword,
]);
function transform(stack, options) {
    var interfaceStack = [];
    var wrapper = [{
            kind: ts.SyntaxKind.ObjectLiteralExpression,
            _kind: 'ObjectLiteralExpression',
            name: options.rootName,
            interfaceCandidate: true,
            body: stack
        }];
    var interfaces = getInterfaces(wrapper);
    return collapse_interfaces_1.collapseInterfaces(interfaces);
    function createOne(node) {
        var thisMembers = getMembers(node.body);
        var item = ts.createNode(ts.SyntaxKind.InterfaceDeclaration);
        item.name = ts.createIdentifier(newInterfaceName(node));
        item.members = ts.createNodeArray(thisMembers, false);
        return item;
    }
    function getInterfaces(nodes) {
        return nodes.reduce(function (acc, node) {
            if (node.kind === ts.SyntaxKind.ObjectLiteralExpression) {
                var newInterface = createOne(node);
                // const asMap = fromJS(newInterface);
                if (node.interfaceCandidate) {
                    return acc.concat([newInterface], getInterfaces(node.body));
                }
                return acc.concat(getInterfaces(node.body));
            }
            if (node.kind === ts.SyntaxKind.ArrayLiteralExpression) {
                var decorated = node.body.map(function (arrayNode) {
                    arrayNode.name = getArrayItemName(node.name);
                    return arrayNode;
                });
                var other = getInterfaces(decorated);
                return acc.concat(other);
            }
            return acc;
        }, []);
    }
    function getMembers(stack) {
        var members = stack.map(function (node) {
            switch (node.kind) {
                case ts.SyntaxKind.FalseKeyword:
                case ts.SyntaxKind.TrueKeyword: {
                    var item = namedProp({ name: node.name });
                    item.type = ts.createNode(ts.SyntaxKind.BooleanKeyword);
                    return item;
                }
                case ts.SyntaxKind.StringLiteral: {
                    var item = namedProp({ name: node.name });
                    item.type = ts.createNode(ts.SyntaxKind.StringKeyword);
                    return item;
                }
                case ts.SyntaxKind.NullKeyword: {
                    var item = namedProp({ name: node.name });
                    item.type = ts.createNode(ts.SyntaxKind.NullKeyword);
                    return item;
                }
                case ts.SyntaxKind.NumericLiteral: {
                    var item = namedProp({ name: node.name });
                    item.type = ts.createNode(ts.SyntaxKind.NumberKeyword);
                    return item;
                }
                case ts.SyntaxKind.ObjectLiteralExpression: {
                    if (node.interfaceCandidate) {
                        var item = namedProp({ name: node.name });
                        item.type = ts.createTypeReferenceNode(newInterfaceName(node), undefined);
                        return item;
                    }
                    else {
                        var item = namedProp({ name: node.name });
                        item.type = ts.createTypeLiteralNode(getMembers(node.body));
                        return item;
                    }
                }
                case ts.SyntaxKind.ArrayLiteralExpression: {
                    if (node.body.length) {
                        var item = namedProp({ name: node.name });
                        var elements = getArrayElementsType(node);
                        item.type = ts.createArrayTypeNode(elements);
                        return item;
                    }
                    else {
                        var item = namedProp({ name: node.name });
                        var anyNode = ts.createNode(ts.SyntaxKind.AnyKeyword);
                        item.type = ts.createArrayTypeNode(anyNode);
                        return item;
                    }
                }
            }
        });
        return members;
    }
    function getArrayElementsType(node) {
        var kinds = immutable_1.Set(node.body.map(function (x) { return x.kind; }));
        if (kinds.size === 1) {
            var kind = kinds.first();
            switch (kind) {
                case ts.SyntaxKind.NullKeyword:
                case ts.SyntaxKind.StringLiteral:
                case ts.SyntaxKind.TrueKeyword:
                case ts.SyntaxKind.FalseKeyword:
                case ts.SyntaxKind.NumericLiteral:
                    return ts.createNode(kindMap[kind]);
                case ts.SyntaxKind.ObjectLiteralExpression:
                    var item = ts.createTypeReferenceNode(getArrayInterfaceItemName(node.name), undefined);
                    return item;
                default: return ts.createNode(ts.SyntaxKind.AnyKeyword);
            }
        }
        else if (kinds.size === 2) {
            if (kinds.has(ts.SyntaxKind.TrueKeyword) && kinds.has(ts.SyntaxKind.FalseKeyword)) {
                return ts.createNode(ts.SyntaxKind.BooleanKeyword);
            }
        }
        // console.log(node.body);
        if (kinds.every(function (kind) { return safeUnions.has(kind); })) {
            // console.log(node.body);
            var types = kinds.map(function (x) {
                return ts.createNode(kindMap[x]);
            }).toJS();
            var item = ts.createNode(ts.SyntaxKind.ParenthesizedType);
            item.type = ts.createUnionOrIntersectionTypeNode(ts.SyntaxKind.UnionType, types);
            return item;
        }
        else {
            console.log('Not creating union as this array contains mixed complexr types');
        }
        return ts.createNode(ts.SyntaxKind.AnyKeyword);
    }
    function newInterfaceName(node) {
        var base = node.name[0].toUpperCase() + node.name.slice(1);
        if (options.prefix) {
            return options.prefix + base;
        }
        return base;
    }
    function upper(string) {
        return string[0].toUpperCase() + string.slice(1);
    }
    function pascalCase(input) {
        return startCase(toLower(input)).replace(/ /g, '');
    }
    function getArrayInterfaceItemName(input) {
        if (options.prefix) {
            return pascalCase(options.prefix + "_" + input + "_Item");
        }
        return pascalCase(input + "_Item");
    }
    function getArrayItemName(input) {
        return pascalCase(input + "_Item");
    }
    // function getArrayInterfaceItemName(input) {
    //     return pascalCase(`I_${input}_Item`)
    // }
}
exports.transform = transform;
var _b;
