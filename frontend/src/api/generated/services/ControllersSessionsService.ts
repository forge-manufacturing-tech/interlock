/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { Params } from '../models/Params';
import type { SessionResponse } from '../models/SessionResponse';
import type { CancelablePromise } from '../core/CancelablePromise';
import { OpenAPI } from '../core/OpenAPI';
import { request as __request } from '../core/request';
export class ControllersSessionsService {
    /**
     * @param projectId
     * @returns SessionResponse List sessions
     * @throws ApiError
     */
    public static list(
        projectId?: string | null,
    ): CancelablePromise<Array<SessionResponse>> {
        return __request(OpenAPI, {
            method: 'GET',
            url: '/api/sessions',
            query: {
                'project_id': projectId,
            },
        });
    }
    /**
     * @param requestBody
     * @returns SessionResponse Session created
     * @throws ApiError
     */
    public static add(
        requestBody: Params,
    ): CancelablePromise<SessionResponse> {
        return __request(OpenAPI, {
            method: 'POST',
            url: '/api/sessions',
            body: requestBody,
            mediaType: 'application/json',
        });
    }
    /**
     * @param id Session ID
     * @returns any Session deleted
     * @throws ApiError
     */
    public static remove(
        id: string,
    ): CancelablePromise<any> {
        return __request(OpenAPI, {
            method: 'DELETE',
            url: '/api/sessions/{id}',
            path: {
                'id': id,
            },
            errors: {
                404: `Session not found`,
            },
        });
    }
    /**
     * @param id Session ID
     * @returns SessionResponse Get session
     * @throws ApiError
     */
    public static getOne(
        id: string,
    ): CancelablePromise<SessionResponse> {
        return __request(OpenAPI, {
            method: 'GET',
            url: '/api/sessions/{id}',
            path: {
                'id': id,
            },
            errors: {
                404: `Session not found`,
            },
        });
    }
    /**
     * @param id Session ID
     * @param requestBody
     * @returns SessionResponse Session updated
     * @throws ApiError
     */
    public static update(
        id: string,
        requestBody: Params,
    ): CancelablePromise<SessionResponse> {
        return __request(OpenAPI, {
            method: 'PUT',
            url: '/api/sessions/{id}',
            path: {
                'id': id,
            },
            body: requestBody,
            mediaType: 'application/json',
            errors: {
                404: `Session not found`,
            },
        });
    }
}
