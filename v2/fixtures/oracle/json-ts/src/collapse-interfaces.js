"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var ts = require("typescript");
var immutable_1 = require("immutable");
var transformer_1 = require("./transformer");
function membersMatch(first, second) {
    if (first.kind !== second.kind) {
        return false;
    }
    if (first.name.text !== second.name.text) {
        return false;
    }
    if (first.type.kind !== second.type.kind) {
        return false;
    }
    if (first.type.kind === ts.SyntaxKind.ArrayType && second.type.kind === ts.SyntaxKind.ArrayType) {
        if (first.type.elementType.kind !== second.type.elementType.kind) {
            return false;
        }
    }
    return true;
}
function isEmptyArrayType(member) {
    if (member.type.kind === ts.SyntaxKind.ArrayType) {
        if (member.type.elementType.kind === ts.SyntaxKind.AnyKeyword) {
            return true;
        }
    }
    return false;
}
function collapseInterfaces(interfaces) {
    return interfaces.reduce(function (acc, current) {
        var currentName = current.name.text;
        var currentMemberNames = immutable_1.Set(current.members.map(function (x) { return (x.name || x.label).text; }));
        var matchingInterfaceIndex = acc.findIndex(function (x) { return (x.name || x.label).text === currentName; });
        if (matchingInterfaceIndex > -1) {
            return acc
                .map(function (int, index) {
                if (index === matchingInterfaceIndex) {
                    var prevMemberNames = immutable_1.Set(int.members.map(function (x) { return (x.name || x.label).text; }));
                    // if the current interface has less props than a previous one
                    // we need to back-track and make the previous one optional
                    if (currentMemberNames.size < prevMemberNames.size) {
                        // elements that existed before, but not in the current
                        var missing = int.members.filter(function (x) { return !currentMemberNames.has(x.name.text); });
                        missing.forEach(function (mem) {
                            mem.questionToken = ts.createNode(ts.SyntaxKind.QuestionToken);
                        });
                    }
                    if (currentMemberNames.has(int.name.text)) {
                        console.log('exists in both, maybe union', int.name.text);
                    }
                    else {
                        // console.log('incoming current does not exist in prev');
                        var existinMemberNames = immutable_1.Set(int.members.map(function (x) { return x.name.text; }));
                        var newMembers_1 = int.members.slice();
                        // Loop over incoming current members
                        current.members.forEach(function (mem) {
                            var existingIndex = int.members.findIndex(function (x) { return x.name.text === mem.name.text; });
                            var existingMember = int.members[existingIndex];
                            // Here, the current member does NOT already exist in this
                            // interface, so we add it, but as optional
                            if (!existingMember) {
                                mem.questionToken = ts.createNode(ts.SyntaxKind.QuestionToken);
                                newMembers_1.push(mem);
                            }
                            else {
                                // here it exists in both, are the types the same?
                                // console.log(ts.SyntaxKind[mem.type.kind]);
                                // console.log(existingMember.kind, mem.kind);
                                if (membersMatch(existingMember, mem)) {
                                    return;
                                }
                                else {
                                    var updatedMember = transformer_1.namedProp({ name: existingMember.name.text });
                                    // const exists  = existingMember.type.types.some(x => x.kind === mem.kind);
                                    // already a union, so just push a new type
                                    if (existingMember.type.kind === ts.SyntaxKind.UnionType) {
                                        var asSet = immutable_1.Set(existingMember.type.types.map(function (x) { return x.kind; }));
                                        if (!asSet.contains(mem.type.kind)) {
                                            existingMember.type.types.push(mem.type);
                                            newMembers_1[existingIndex] = existingMember;
                                        }
                                    }
                                    else {
                                        // was this previously marked as an empty array? eg: any[]
                                        // if so & the next item is NOT, then we can ignore the any[]
                                        if (isEmptyArrayType(existingMember) && !isEmptyArrayType(mem)) {
                                            updatedMember.type = ts.createNode(ts.SyntaxKind.ArrayType);
                                            updatedMember.type.elementType = mem.type.elementType;
                                            newMembers_1[existingIndex] = updatedMember;
                                        }
                                        else {
                                            // If the INCOMING member type is an empty array, but we already have an array element with items, we bail
                                            if (isEmptyArrayType(mem) && existingMember.type.kind === ts.SyntaxKind.ArrayType && (!isEmptyArrayType(existingMember))) {
                                                return;
                                            }
                                            var memberNodes = [existingMember.type, mem.type];
                                            updatedMember.type = ts.createUnionOrIntersectionTypeNode(ts.SyntaxKind.UnionType, memberNodes);
                                            newMembers_1[existingIndex] = updatedMember;
                                        }
                                    }
                                }
                                // console.log(ts.compareDataObjects(existingMember, mem));
                            }
                        });
                        int.members = newMembers_1;
                        return int;
                    }
                }
                return int;
            });
        }
        else {
            // console.log('Agressive merge here?')
        }
        return acc.concat(current);
    }, []);
}
exports.collapseInterfaces = collapseInterfaces;
