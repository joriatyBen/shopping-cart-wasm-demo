import { HandleRequest, HttpRequest, HttpResponse } from '@fermyon/spin-sdk';
import 'urlpattern-polyfill';

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

    return {
        status: 200,
        headers: { 'content-type': 'text/plain' },
        body: `Hello from TS-SDK`,
    };
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
    return {
        status: 200,
        headers: { 'content-type': 'text/plain' },
        body: `GET /carts/${cartId}`,
    };
}

async function cartsItemsGET(request: HttpRequest, cartId: number): Promise<HttpResponse> {
    return {
        status: 200,
        headers: { 'content-type': 'text/plain' },
        body: `GET /carts/${cartId}/items`,
    };
}

async function cartsItemsPOST(request: HttpRequest, cartId: number): Promise<HttpResponse> {
    return {
        status: 200,
        headers: { 'content-type': 'text/plain' },
        body: `POST /carts/${cartId}/items`,
    };
}

async function cartsItemsPATCH(request: HttpRequest, cartId: number): Promise<HttpResponse> {
    return {
        status: 200,
        headers: { 'content-type': 'text/plain' },
        body: `PATCH /carts/${cartId}/items`,
    };
}

async function cartsItemsDELETE(request: HttpRequest, cartId: number): Promise<HttpResponse> {
    return {
        status: 200,
        headers: { 'content-type': 'text/plain' },
        body: `DELETE /carts/${cartId}/items`,
    };
}

async function cartsItemDELETE(request: HttpRequest, cartId: number, itemId: number): Promise<HttpResponse> {
    return {
        status: 200,
        headers: { 'content-type': 'text/plain' },
        body: `DELETE /carts/${cartId}/items/${itemId}`,
    };
}

function responseBadRequest(message: string): HttpResponse {
    return {
        status: 400,
        body: message,
    };
}

function parseId(serializedId: string | undefined): number | undefined {
    if (serializedId === undefined) return undefined;
    if (!/^\d+$/.test(serializedId)) return undefined;

    return parseInt(serializedId, 10);
}
