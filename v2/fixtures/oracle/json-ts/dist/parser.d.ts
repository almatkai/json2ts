import * as ts from 'typescript';
import { JsonTsOptions } from "./index";
export interface ParsedNode {
    kind: ts.SyntaxKind;
    _kind: string;
    name?: string;
    value?: any;
    body?: ParsedNode[];
    interfaceCandidate?: boolean;
}
export declare function parse(string: any, options: JsonTsOptions): {
    stack: any[];
    inputKind: ts.SyntaxKind;
};
