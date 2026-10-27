import * as ts from 'typescript';
import { ParsedNode } from "./parser";
import { Set as ImmutableSet } from "immutable";
import { JsonTsOptions } from "./index";
export declare const log: (input: any) => void;
export interface MemberNode {
    types: ImmutableSet<string>;
    members: MemberNode[];
    name: string;
    optional: boolean;
}
export interface InterfaceNode {
    name: string;
    original: string;
    members: MemberNode[];
}
export declare const kindMap: {
    [x: number]: ts.SyntaxKind;
};
export declare function namedProp(member: any): any;
export declare function transform(stack: ParsedNode[], options: JsonTsOptions): InterfaceNode[];
