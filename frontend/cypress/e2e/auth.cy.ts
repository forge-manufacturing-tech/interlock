describe('Authentication Flow', () => {
    beforeEach(() => {
        cy.visit('/login')
    })

    it('should display login page', () => {
        cy.contains('Interlock').should('be.visible')
        cy.contains('Sign in to your account').should('be.visible')
        cy.get('input[type="email"]').should('be.visible')
        cy.get('input[type="password"]').should('be.visible')
    })

    it('should toggle between login and register', () => {
        cy.contains("Don't have an account?").click()
        cy.contains('Create your account').should('be.visible')
        cy.get('input[type="text"]').should('be.visible') // Name field only in register

        cy.contains('Already have an account?').click()
        cy.contains('Sign in to your account').should('be.visible')
    })

    it('should show error on invalid login', () => {
        cy.get('input[type="email"]').type('nonexistent@example.com')
        cy.get('input[type="password"]').type('wrongpassword')
        cy.get('button[type="submit"]').click()

        // Wait for error element to appear (API might be slow)
        cy.get('.bg-red-500\\/10', { timeout: 20000 }).should('be.visible')
        // Should still be on login page
        cy.url().should('include', '/login')
    })

    it('should complete full registration and login flow', () => {
        const email = `test-${Date.now()}@example.com`
        const password = 'TestPassword123!'
        const name = 'Cypress Test User'

        // Register
        cy.contains("Don't have an account?").click()
        cy.get('input[type="text"]').type(name)
        cy.get('input[type="email"]').type(email)
        cy.get('input[type="password"]').type(password)
        cy.get('button[type="submit"]').click()

        // Make sure no error appeared
        cy.get('.bg-red-500\\/10').should('not.exist')

        // Wait a bit for React state to update
        

        // Should redirect to projects page (might take a moment)
        cy.url({ timeout: 20000 }).should('eq', 'http://localhost:3000/')
        cy.contains('Projects', { timeout: 20000 }).should('be.visible')
        cy.contains(email).should('be.visible')
    })
})
