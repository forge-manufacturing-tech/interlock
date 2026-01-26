/// <reference types="cypress" />

// Custom commands for authentication
Cypress.Commands.add('login', (email: string, password: string) => {
    cy.visit('/login')
    cy.get('input[type="email"]').type(email)
    cy.get('input[type="password"]').type(password)
    cy.get('button[type="submit"]').click()
    cy.url().should('eq', 'http://localhost:3000/')
})

Cypress.Commands.add('register', (name: string, email: string, password: string) => {
    cy.visit('/login')
    cy.contains("Don't have an account?").click()
    cy.get('input[type="text"]').type(name)
    cy.get('input[type="email"]').type(email)
    cy.get('input[type="password"]').type(password)
    cy.get('button[type="submit"]').click()
    cy.url().should('eq', 'http://localhost:3000/')
})

declare global {
    namespace Cypress {
        interface Chainable {
            login(email: string, password: string): Chainable<void>
            register(name: string, email: string, password: string): Chainable<void>
        }
    }
}

export { }
