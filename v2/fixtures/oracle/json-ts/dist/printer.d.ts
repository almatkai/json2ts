import * as ts from 'typescript';
import { JsonTsOptions } from "./index";
export declare function print(interfaceNodes: any, inputKind: ts.SyntaxKind, options: JsonTsOptions): string;
export declare function printLiteral(node: any, kind: any, options: any): string;
