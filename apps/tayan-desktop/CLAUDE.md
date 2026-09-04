# Claude MCP Server Documentation

This project uses multiple MCP servers to provide comprehensive documentation access.

## Available MCP Servers

### Svelte MCP Server

You have access to comprehensive Svelte 5 and SvelteKit documentation. Here's how to use the available tools effectively:

#### Available Tools

**1. list-sections**

- Use this FIRST to discover all available documentation sections
- Returns a structured list with titles, use_cases, and paths
- When asked about Svelte or SvelteKit topics, ALWAYS use this tool at the start to find relevant sections

**2. get-documentation**

- Retrieves full documentation content for specific sections
- Accepts single or multiple sections
- After calling list-sections, you MUST analyze the returned documentation (especially the use_cases field) and then fetch ALL relevant sections for the user's task

**3. svelte-autofixer**

- Analyzes Svelte code and returns issues and suggestions
- You MUST use this tool whenever writing Svelte code before sending it to the user
- Keep calling it until no issues or suggestions are returned

**4. playground-link**

- Generates a Svelte Playground link with the provided code
- After completing the code, ask the user if they want a playground link
- NEVER call this if code was written to files in their project

### Flowbite-Svelte MCP Server

You have access to comprehensive Flowbite-Svelte component documentation. Here's how to use the available tools effectively:

#### Available Tools

**1. findComponent**

- Use this FIRST to discover components by name or category
- Returns component information including the documentation path
- When asked about Flowbite-Svelte components, ALWAYS use this tool to locate the correct component before fetching documentation
- Example queries: 'Button', 'CardPlaceholder', 'form checkbox'

**2. getComponentList**

- Lists all available Flowbite-Svelte components with their categories
- Use this to discover what components are available or to help users explore component options

**3. getComponentDoc**

- Retrieves full documentation content for a specific component
- Accepts the component path found using findComponent
- After calling findComponent, use this tool to fetch complete documentation including usage examples, props, and best practices

**4. searchDocs**

- Performs full-text search across all Flowbite-Svelte documentation
- Use this when you need to find specific information that might span multiple components or when the user asks about features or patterns

## Workflow Guidelines

### When building Svelte components with Flowbite-Svelte

1. **Start with Svelte documentation**: Use `list-sections` to understand which Svelte concepts are needed
2. **Fetch relevant Svelte docs**: Use `get-documentation` to get all necessary Svelte sections
3. **Find Flowbite-Svelte components**: Use `findComponent` to locate the UI components needed
4. **Get component details**: Use `getComponentDoc` to fetch usage examples and props
5. **Write the code**: Combine Svelte patterns with Flowbite-Svelte components
6. **Validate the code**: Use `svelte-autofixer` to check for issues
7. **Offer playground**: Ask if the user wants a playground link (only if not writing to files)

### Best Practices

- Always prioritize Svelte 5 runes and modern patterns
- Use Flowbite-Svelte components for consistent UI design
- Validate all code with svelte-autofixer before delivering
- Keep documentation lookups efficient by fetching multiple sections at once

<!-- ALP-LAB:BEGIN -->
## Alp Lab orchestrator (managed)
Operate as the always-on Opus orchestrator. Invoke the `alp-lab:alp-orchestrator`
skill via the Skill tool (a relative `skills/...` path resolves nowhere from a
project checkout — the plugin lives outside the project).
Standing ultracode authorization: you MAY call the Workflow tool to fan out large
file-disjoint batches across the tiered alp-* agents (no per-session re-ask); the
bench stays serial and out of any workflow.

## Data fidelity (managed)
Output style is caveman's job, not this plugin's — see the caveman plugin. These
are not style and no style switches them off.

Verbatim always — registers, hex, bit fields, addresses, I2C addresses, pin
names, SKUs, part numbers, hw_rev, diagnostic codes, error strings, probe/PSU
serials, USB paths, labgrid places, IP:port, voltages, clock/baud rates, DT
nodes, Kconfig symbols, commands, paths. A rounded number or dropped digit
flashes the wrong module or powers a board off-rail. Ordered bench steps keep
their sequence words. Risk outranks brevity: failures, hardware-damage and
data-loss caveats, and corrections are never unrequested.
<!-- ALP-LAB:END -->
