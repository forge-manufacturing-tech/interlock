describe('Sessions Management', () => {
    const password = 'TestPassword123!'
    let projectName: string

    beforeEach(() => {
        // Register with unique email for this test suite
        const email = `test-sessions-${Date.now()}@example.com`
        projectName = `Session Test Project ${Date.now()}`

        // Setup: Register, login, and create a project
        cy.visit('/login')
        cy.contains("Don't have an account?").click()
        cy.get('input[type="text"]').type('Test User')
        cy.get('input[type="email"]').type(email)
        cy.get('input[type="password"]').type(password)
        cy.get('button[type="submit"]').click()

        // Wait for redirect
        cy.url({ timeout: 20000 }).should('eq', 'http://localhost:3000/')
        cy.contains('Projects', { timeout: 20000 }).should('be.visible')
        cy.contains(/no projects found/i, { timeout: 10000 })

        cy.contains('+ New Project', { timeout: 10000 }).click()
        cy.get('input[type="text"]').first().type(projectName)
        cy.contains('button', 'Create').click()
        cy.contains(projectName, { timeout: 10000 }).click()

        // Wait for sessions page to load
        cy.url({ timeout: 10000 }).should('include', '/projects/')
        cy.contains(/no sessions yet/i, { timeout: 10000 })
    })

    it('should display empty sessions page', () => {
        cy.contains(/no sessions yet/i).should('be.visible')
    })

    it('should create a new session', () => {
        const sessionTitle = `Test Session ${Date.now()}`

        cy.contains('+ New Session', { timeout: 10000 }).should('be.visible').click()
        cy.contains('Create New Session', { timeout: 10000 }).should('be.visible')
        cy.get('input[required]').type(sessionTitle)
        cy.contains('button', 'Create').click()

        // Session should appear in list
        cy.contains(sessionTitle, { timeout: 10000 }).should('be.visible')
    })

    it('should select and display session content', () => {
        const sessionTitle = `Selectable Session ${Date.now()}`

        // Create session
        cy.contains('+ New Session', { timeout: 10000 }).click()
        cy.get('input[required]').type(sessionTitle)
        cy.contains('button', 'Create').click()

        // Click on session
        cy.contains(sessionTitle, { timeout: 10000 }).click()

        // Should display session title in content area
        cy.contains(sessionTitle).should('be.visible')
    })

    it('should navigate back to projects', () => {
        cy.contains('Back', { timeout: 10000 }).click()
        cy.url({ timeout: 10000 }).should('eq', 'http://localhost:3000/')
        cy.contains('Projects').should('be.visible')
    })
})
