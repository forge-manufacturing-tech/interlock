/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { CreateProjectParams } from '../models/CreateProjectParams';
import type { ProjectResponse } from '../models/ProjectResponse';
import type { ShareProjectParams } from '../models/ShareProjectParams';
import type { UpdateProjectParams } from '../models/UpdateProjectParams';
import type { UserSearchResponse } from '../models/UserSearchResponse';
import type { CancelablePromise } from '../core/CancelablePromise';
import { OpenAPI } from '../core/OpenAPI';
import { request as __request } from '../core/request';
export class ControllersProjectsService {
    /**
     * @returns ProjectResponse List all projects for current user
     * @throws ApiError
     */
    public static list(): CancelablePromise<Array<ProjectResponse>> {
        return __request(OpenAPI, {
            method: 'GET',
            url: '/api/projects',
        });
    }
    /**
     * @param requestBody
     * @returns ProjectResponse Project created
     * @throws ApiError
     */
    public static create(
        requestBody: CreateProjectParams,
    ): CancelablePromise<ProjectResponse> {
        return __request(OpenAPI, {
            method: 'POST',
            url: '/api/projects',
            body: requestBody,
            mediaType: 'application/json',
        });
    }
    /**
     * @param id Project ID
     * @returns any Project deleted
     * @throws ApiError
     */
    public static remove(
        id: string,
    ): CancelablePromise<any> {
        return __request(OpenAPI, {
            method: 'DELETE',
            url: '/api/projects/{id}',
            path: {
                'id': id,
            },
            errors: {
                404: `Project not found`,
            },
        });
    }
    /**
     * @param id Project ID
     * @returns ProjectResponse Get project
     * @throws ApiError
     */
    public static getOne(
        id: string,
    ): CancelablePromise<ProjectResponse> {
        return __request(OpenAPI, {
            method: 'GET',
            url: '/api/projects/{id}',
            path: {
                'id': id,
            },
            errors: {
                403: `Unauthorized`,
                404: `Project not found`,
            },
        });
    }
    /**
     * @param id Project ID
     * @param requestBody
     * @returns ProjectResponse Project updated
     * @throws ApiError
     */
    public static update(
        id: string,
        requestBody: UpdateProjectParams,
    ): CancelablePromise<ProjectResponse> {
        return __request(OpenAPI, {
            method: 'PUT',
            url: '/api/projects/{id}',
            path: {
                'id': id,
            },
            body: requestBody,
            mediaType: 'application/json',
            errors: {
                404: `Project not found`,
            },
        });
    }
    /**
     * @param id Project ID
     * @param requestBody
     * @returns any Project shared
     * @throws ApiError
     */
    public static share(
        id: string,
        requestBody: ShareProjectParams,
    ): CancelablePromise<any> {
        return __request(OpenAPI, {
            method: 'POST',
            url: '/api/projects/{id}/share',
            path: {
                'id': id,
            },
            body: requestBody,
            mediaType: 'application/json',
            errors: {
                400: `User already in project`,
                404: `Project or User not found`,
            },
        });
    }
    /**
     * @param q Search query
     * @returns UserSearchResponse Search results
     * @throws ApiError
     */
    public static searchUsers(
        q: string,
    ): CancelablePromise<Array<UserSearchResponse>> {
        return __request(OpenAPI, {
            method: 'GET',
            url: '/api/users/search',
            query: {
                'q': q,
            },
        });
    }
}
