/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { ChatParams } from '../models/ChatParams';
import type { MessageResponse } from '../models/MessageResponse';
import type { CancelablePromise } from '../core/CancelablePromise';
import { OpenAPI } from '../core/OpenAPI';
import { request as __request } from '../core/request';
export class ControllersChatService {
    /**
     * @param sessionId Session ID
     * @param requestBody
     * @returns MessageResponse Chat response
     * @throws ApiError
     */
    public static chat(
        sessionId: string,
        requestBody: ChatParams,
    ): CancelablePromise<MessageResponse> {
        return __request(OpenAPI, {
            method: 'POST',
            url: '/api/sessions/{session_id}/chat',
            path: {
                'session_id': sessionId,
            },
            body: requestBody,
            mediaType: 'application/json',
            errors: {
                404: `Session not found`,
            },
        });
    }
    /**
     * @param sessionId Session ID
     * @returns any Messages cleared
     * @throws ApiError
     */
    public static clearMessages(
        sessionId: string,
    ): CancelablePromise<any> {
        return __request(OpenAPI, {
            method: 'DELETE',
            url: '/api/sessions/{session_id}/messages',
            path: {
                'session_id': sessionId,
            },
        });
    }
    /**
     * @param sessionId Session ID
     * @returns MessageResponse List messages
     * @throws ApiError
     */
    public static listMessages(
        sessionId: string,
    ): CancelablePromise<Array<MessageResponse>> {
        return __request(OpenAPI, {
            method: 'GET',
            url: '/api/sessions/{session_id}/messages',
            path: {
                'session_id': sessionId,
            },
        });
    }
}
