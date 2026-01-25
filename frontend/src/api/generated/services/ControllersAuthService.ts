/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { LoginParams } from '../models/LoginParams';
import type { LoginResponse } from '../models/LoginResponse';
import type { RegisterParams } from '../models/RegisterParams';
import type { CancelablePromise } from '../core/CancelablePromise';
import { OpenAPI } from '../core/OpenAPI';
import { request as __request } from '../core/request';
export class ControllersAuthService {
    /**
     * Creates a user login and returns a token
     * @param requestBody
     * @returns LoginResponse Login successful
     * @throws ApiError
     */
    public static login(
        requestBody: LoginParams,
    ): CancelablePromise<LoginResponse> {
        return __request(OpenAPI, {
            method: 'POST',
            url: '/api/auth/login',
            body: requestBody,
            mediaType: 'application/json',
            errors: {
                401: `Unauthorized`,
            },
        });
    }
    /**
     * Register function creates a new user with the given parameters and sends a
     * welcome email to the user
     * @param requestBody
     * @returns any User registered successfully
     * @throws ApiError
     */
    public static register(
        requestBody: RegisterParams,
    ): CancelablePromise<any> {
        return __request(OpenAPI, {
            method: 'POST',
            url: '/api/auth/register',
            body: requestBody,
            mediaType: 'application/json',
            errors: {
                400: `Bad Request`,
            },
        });
    }
}
