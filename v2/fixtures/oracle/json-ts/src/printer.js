"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var ts = require("typescript");
function print(interfaceNodes, options) {
    var result = ts.createSourceFile('module', '');
    var printer = ts.createPrinter({
        newLine: ts.NewLineKind.LineFeed,
    });
    if (options.flow) {
        var modified = interfaceNodes.map(function (x) {
            var newNode = ts.createNode(ts.SyntaxKind.TypeAliasDeclaration);
            newNode.modifiers = [ts.createToken(ts.SyntaxKind.ExportKeyword)];
            newNode.type = ts.createTypeLiteralNode(x.members);
            newNode.name = x.name;
            return newNode;
        });
        return modified.map(function (x) {
            return printer.printNode(ts.EmitHint.Unspecified, x, result);
        }).join('\n') + '\n';
    }
    if (options.namespace) {
        interfaceNodes.forEach(function (x) {
            x.modifiers = [ts.createToken(ts.SyntaxKind.ExportKeyword)];
        });
        var ns = ts.createModuleDeclaration(undefined, [ts.createToken(ts.SyntaxKind.DeclareKeyword)], ts.createIdentifier(options.namespace), ts.createModuleBlock(interfaceNodes), ts.NodeFlags.Namespace);
        return printer.printNode(ts.EmitHint.Unspecified, ns, result) + '\n';
    }
    return interfaceNodes.map(function (x) {
        return printer.printNode(ts.EmitHint.Unspecified, x, result);
    }).join('\n') + '\n';
}
exports.print = print;
function wrapper(blocks, options) {
    if (options.namespace) {
        var lines = [
            "declare namespace " + options.namespace + " {"
        ].concat(blocks.split('\n').map(function (x) { return "  " + x; }), [
            "}",
        ]);
        return lines.join('\n');
    }
    return blocks;
}
