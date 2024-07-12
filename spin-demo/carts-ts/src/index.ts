import { HandleRequest, HttpRequest, HttpResponse, Pg, Config } from '@fermyon/spin-sdk';
import Ajv, { JTDDataType, Schema, ValidateFunction } from 'ajv/dist/jtd';
import 'urlpattern-polyfill';

// This is not exported from the SDK, so redefine it here
type RdbmsParam = null | boolean | string | number | ArrayBuffer;

// The SDK typings are wrong for this. so we redefine it
interface QueryResult {
    columns: Array<string>;
    rows: Array<Array<RdbmsParam>>;
}

interface ParseBodyResultOK<T> {
    ok: true;
    body: T;
}

interface ParseBodyResultError {
    ok: false;
    error: string;
}

type ParseBodyResult<T> = ParseBodyResultOK<T> | ParseBodyResultError;

const cartItemSchema = {
    properties: {
        itemId: { type: 'uint32' },
        quantity: { type: 'uint32' },
        price: { type: 'float64' },
    },
} as const;

const cartItemPatchSchema = {
    properties: {
        itemId: { type: 'uint32' },
    },
    optionalProperties: {
        quantity: { type: 'uint32' },
        price: { type: 'float64' },
    },
} as const;

type CartItem = JTDDataType<typeof cartItemSchema>;
type CartItemPatch = JTDDataType<typeof cartItemPatchSchema>;

const validators = new Map();
const ajv = new Ajv();

// ajv.validate is deadly slow in QuickJS (~seconds!), so we precompile the validators.
// The build process preevaluates and snapshots the code running in QuickJS, so the compilation
// happens during build and does not incur runtime overhead.
validators.set(cartItemSchema, ajv.compile<CartItem>(cartItemSchema));
validators.set(cartItemPatchSchema, ajv.compile<CartItemPatch>(cartItemPatchSchema));

export const handleRequest: HandleRequest = async function (request: HttpRequest): Promise<HttpResponse> {
    const now = Date.now();

    const response = await dispatch(request);

    if (!response.headers) response.headers = {};
    response.headers['X-Processing-Time-Milliseconds'] = Math.round(Date.now() - now) + '';

    return response;
};

async function dispatch(request: HttpRequest): Promise<HttpResponse> {
    const patternCarts = new URLPattern({ pathname: '/carts-ts/:cartId' });
    const patternCartsItems = new URLPattern({ pathname: '/carts-ts/:cartId/items' });
    const patternCartsItem = new URLPattern({ pathname: '/carts-ts/:cartId/items/:itemId' });

    const url = request.headers['spin-full-url'];

    const cartsMatch = patternCarts.exec(url);
    if (cartsMatch) {
        return handleCarts(request, parseId(cartsMatch.pathname.groups['cartId']));
    }

    const cartsItemsMatch = patternCartsItems.exec(url);
    if (cartsItemsMatch) {
        return handleCartsItems(request, parseId(cartsItemsMatch.pathname.groups['cartId']));
    }

    const cartsItemMatch = patternCartsItem.exec(url);
    if (cartsItemMatch) {
        return handleCartsItem(
            request,
            parseId(cartsItemMatch.pathname.groups['cartId']),
            parseId(cartsItemMatch.pathname.groups['itemId'])
        );
    }

    return responseNotFound();
}

async function handleCarts(request: HttpRequest, cartId: number | undefined): Promise<HttpResponse> {
    if (cartId === undefined) return responseBadRequest('invalid id');
    if (request.method !== 'GET') return responseBadRequest('unsupported method');

    return cartsGET(request, cartId);
}

async function handleCartsItems(request: HttpRequest, cartId: number | undefined): Promise<HttpResponse> {
    if (cartId === undefined) return responseBadRequest('invalid id');

    switch (request.method) {
        case 'GET':
            return cartsItemsGET(request, cartId);

        case 'POST':
            return cartsItemsPOST(request, cartId);

        case 'PATCH':
            return cartsItemsPATCH(request, cartId);

        case 'DELETE':
            return cartsItemsDELETE(request, cartId);

        default:
            return responseBadRequest('unsupported method');
    }
}

async function handleCartsItem(
    request: HttpRequest,
    cartId: number | undefined,
    itemId: number | undefined
): Promise<HttpResponse> {
    if (cartId === undefined || itemId === undefined) return responseBadRequest('invalid id');

    if (request.method !== 'DELETE') return responseBadRequest('unsupported method');

    return cartsItemDELETE(request, cartId, itemId);
}

async function cartsGET(request: HttpRequest, cartId: number): Promise<HttpResponse> {
    const queryResult = query('SELECT * FROM cart.cart_items WHERE cart_id = $1', [cartId]);

    return queryResult.rows.length === 0 ? responseNotFound() : responseJson({ customerId: cartId });
}

async function cartsItemsGET(request: HttpRequest, cartId: number): Promise<HttpResponse> {
    const queryResult = query('SELECT item_id, quantity, price FROM cart.cart_items WHERE cart_id = $1', [cartId]);

    if (queryResult.rows.length === 0) return responseNotFound();

    return responseJson(queryResult.rows.map(rowToItem));
}

async function cartsItemsPOST(request: HttpRequest, cartId: number): Promise<HttpResponse> {
    const parsedBody = parseBody(request, cartItemSchema);
    if (!parsedBody.ok) {
        return responseBadRequest(parsedBody.error);
    }

    const item = parsedBody.body;
    // Hack: make sure that the rust <-> QuickJS bridge code properly identifies the parameter
    // as f64
    item.price += Number.EPSILON;

    try {
        execute('INSERT INTO cart.cart_items VALUES($1, $2, $3, $4)', [cartId, item.itemId, item.quantity, item.price]);
    } catch (e) {
        return responseBadRequest('duplicate key');
    }

    return responseJson(item);
}

async function cartsItemsPATCH(request: HttpRequest, cartId: number): Promise<HttpResponse> {
    const parsedBody = parseBody(request, cartItemPatchSchema);
    if (!parsedBody.ok) {
        return responseBadRequest(parsedBody.error);
    }

    const patch = parsedBody.body;
    // Hack: make sure that the rust <-> QuickJS bridge code properly identifies the parameter
    // as f64
    if (patch.price !== undefined) patch.price += Number.EPSILON;

    const params: Array<RdbmsParam> = [cartId, patch.itemId];
    const mutations: Array<string> = [];

    if (patch.quantity !== undefined) {
        params.push(patch.quantity);
        mutations.push(`quantity = $${params.length}`);
    }

    if (patch.price !== undefined) {
        params.push(patch.price);
        mutations.push(`price = $${params.length}`);
    }

    if (mutations.length === 0) return responseEmpty();

    const sql = `UPDATE cart.cart_items SET ${mutations.join(', ')} WHERE cart_id = $1 AND item_id = $2`;
    execute(sql, params);

    const queryResult = query(
        'SELECT item_id, quantity, price FROM cart.cart_items WHERE cart_id = $1 AND item_id = $2',
        [cartId, patch.itemId]
    );

    if (queryResult.rows.length === 0) return responseNotFound();

    return responseJson(rowToItem(queryResult.rows[0]));
}

async function cartsItemsDELETE(request: HttpRequest, cartId: number): Promise<HttpResponse> {
    const queryResult = query('DELETE FROM cart.cart_items WHERE cart_id = $1 RETURNING *', [cartId]);

    return queryResult.rows.length === 0 ? responseNotFound() : responseEmpty();
}

async function cartsItemDELETE(request: HttpRequest, cartId: number, itemId: number): Promise<HttpResponse> {
    const queryResult = query('DELETE FROM cart.cart_items WHERE cart_id = $1 AND item_id = $2 RETURNING *', [
        cartId,
        itemId,
    ]);

    return queryResult.rows.length === 0 ? responseNotFound() : responseEmpty();
}

function responseBadRequest(message: string): HttpResponse {
    return {
        status: 400,
        body: message,
    };
}

function responseEmpty(): HttpResponse {
    return { status: 200 };
}

function responseJson<T>(payload: T): HttpResponse {
    return {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
    };
}

function responseNotFound(): HttpResponse {
    return {
        status: 404,
    };
}

function parseId(serializedId: string | undefined): number | undefined {
    if (serializedId === undefined) return undefined;
    if (!/^\d+$/.test(serializedId)) return undefined;

    return parseInt(serializedId, 10);
}

function dbUrl(): string {
    const url = Config.get('database_url');

    if (!url) throw new Error('DB_URL not configured');

    return url;
}

function query(sql: string, params: Array<RdbmsParam>): QueryResult {
    return Pg.query(dbUrl(), sql, params);
}

function execute(sql: string, params: Array<RdbmsParam>): void {
    return Pg.execute(dbUrl(), sql, params);
}

function rowToItem(row: Array<RdbmsParam>): CartItem {
    const [id, quantity, price] = row;

    return {
        itemId: id as number,
        quantity: quantity as number,
        price: price as number,
    };
}

// ajv.validate is deadly slow in QuickJS (~seconds!), so we precompile the validators
function validator<T extends Schema>(schema: T): ValidateFunction<JTDDataType<T>> {
    return validators.get(schema);
}

function parseBody<T extends Schema>(request: HttpRequest, schema: T): ParseBodyResult<JTDDataType<T>> {
    if (!request.body) return { ok: false, error: 'missing body' };

    try {
        const decoder = new TextDecoder();
        const body = JSON.parse(decoder.decode(request.body));
        const validate = validator(schema);

        if (!validate(body)) {
            return { ok: false, error: ajv.errorsText(validate.errors) };
        }

        return { ok: true, body };
    } catch (e) {
        console.log(e);
        return { ok: false, error: e + '' };
    }
}
