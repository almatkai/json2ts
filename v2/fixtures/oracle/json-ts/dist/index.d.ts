import { parse } from "./parser";
import { print } from "./printer";
import { transform } from "./transformer";
export interface JsonTsOptions {
    namespace?: string;
    flow?: boolean;
    prefix?: string;
    rootName?: string;
}
export declare const defaults: {
    prefix: string;
    rootName: string;
};
export declare function json2ts(validJsonString: string, options?: JsonTsOptions): string;
export declare function json2tsMulti(validJsonStrings: string[], options?: JsonTsOptions): string;
export { parse, print, transform };
