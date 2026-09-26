import type { paths } from 'admin-api';

/** The methods the admin API's operations use. */
export type HttpMethod = 'get' | 'post' | 'delete' | 'patch';

/** The operation of `method` on `path`, or `never` when the path has none. */
export type Operation<P extends keyof paths, M extends HttpMethod> = NonNullable<paths[P][M]>;

/** The paths with an operation for `method`. */
export type PathsWith<M extends HttpMethod> = {
  [P in keyof paths]: [Operation<P, M>] extends [never] ? never : P;
}[keyof paths];

// Each of these is `undefined` when the operation has no such part, and admits `undefined` when
// the part is optional.
type ParametersOf<Op> = Op extends { parameters: infer Parameters } ? Parameters : never;
type PartOf<Parameters, Key extends 'path' | 'query'> =
  Parameters extends Record<Key, infer Part>
    ? Part
    : Parameters extends Partial<Record<Key, infer Part>>
      ? Part | undefined
      : undefined;
type PathParametersOf<Op> = PartOf<ParametersOf<Op>, 'path'>;
type QueryOf<Op> = PartOf<ParametersOf<Op>, 'query'>;
type RequestBodyOf<Op> = Op extends { requestBody?: infer RequestBody } ? RequestBody : undefined;
type JsonOf<Content> = Content extends { content: { 'application/json': infer Body } }
  ? Body
  : undefined;

/** Keys whose type admits `undefined` become optional. */
type OptionalWhenUndefined<T> = {
  [K in keyof T as undefined extends T[K] ? never : K]: T[K];
} & { [K in keyof T as undefined extends T[K] ? K : never]?: T[K] };

/** What a request of `Op` sends: its path parameters, its query, and its JSON body. */
export type RequestOptions<Op> = OptionalWhenUndefined<{
  path: PathParametersOf<Op>;
  query: QueryOf<Op>;
  body: JsonOf<RequestBodyOf<Op>>;
}>;

/** The options argument of `Op`: optional when every option is. */
export type OptionsArgument<Op> =
  Record<string, never> extends RequestOptions<Op>
    ? [options?: RequestOptions<Op>]
    : [options: RequestOptions<Op>];

type SuccessStatus = 200 | 201 | 204;
type ResponsesOf<Op> = Op extends { responses: infer Responses } ? Responses : never;

/** What a successful request of `Op` returns: its JSON body, or `undefined` without one. */
export type ResponseOf<Op> = JsonOf<ResponsesOf<Op>[SuccessStatus & keyof ResponsesOf<Op>]>;
