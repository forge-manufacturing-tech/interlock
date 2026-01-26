/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { BlobResponse } from '../models/BlobResponse';
import type { CancelablePromise } from '../core/CancelablePromise';
import { OpenAPI } from '../core/OpenAPI';
import { request as __request } from '../core/request';
export class ControllersBlobsService {
    /**
     * @param id Blob ID
     * @returns any Blob deleted
     * @throws ApiError
     */
    public static remove(
        id: string,
    ): CancelablePromise<any> {
        return __request(OpenAPI, {
            method: 'DELETE',
            url: '/api/blobs/{id}',
            path: {
                'id': id,
            },
            errors: {
                404: `Blob not found`,
            },
        });
    }
    /**
     * @param id Blob ID
     * @returns any Download blob
     * @throws ApiError
     */
    public static download(
        id: string,
    ): CancelablePromise<any> {
        return __request(OpenAPI, {
            method: 'GET',
            url: '/api/blobs/{id}/download',
            path: {
                'id': id,
            },
            errors: {
                404: `Blob not found`,
            },
        });
    }
    /**
     * @param sessionId Session ID
     * @returns BlobResponse List blobs
     * @throws ApiError
     */
    public static list(
        sessionId: string,
    ): CancelablePromise<Array<BlobResponse>> {
        return __request(OpenAPI, {
            method: 'GET',
            url: '/api/sessions/{session_id}/blobs',
            path: {
                'session_id': sessionId,
            },
        });
    }
    /**
     * @param sessionId Session ID
     * @returns BlobResponse File uploaded
     * @throws ApiError
     */
    public static upload(
        sessionId: string,
    ): CancelablePromise<BlobResponse> {
        return __request(OpenAPI, {
            method: 'POST',
            url: '/api/sessions/{session_id}/blobs',
            path: {
                'session_id': sessionId,
            },
            errors: {
                404: `Session not found`,
            },
        });
    }
}
