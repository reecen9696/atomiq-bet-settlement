import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const betCreationDuration = new Trend('bet_creation_duration');
const betRetrievalDuration = new Trend('bet_retrieval_duration');

// Test configuration
export const options = {
    stages: [
        { duration: '10s', target: 10 },  // Ramp up to 10 users
        { duration: '20s', target: 10 },  // Stay at 10 users
        { duration: '10s', target: 20 },  // Spike to 20 users
        { duration: '10s', target: 0 },   // Ramp down to 0
    ],
    thresholds: {
        'http_req_duration': ['p(95)<500'], // 95% of requests must complete below 500ms
        'errors': ['rate<0.1'],              // Error rate must be below 10%
        'http_req_failed': ['rate<0.1'],     // Failed requests must be below 10%
    },
};

const BACKEND_URL = __ENV.BACKEND_URL || 'http://localhost:3001';

// Test data
const choices = ['heads', 'tails'];
const stakeAmounts = [
    10_000_000,    // 0.01 SOL (minimum)
    100_000_000,   // 0.1 SOL
    1_000_000_000, // 1 SOL
];

function randomChoice(arr) {
    return arr[Math.floor(Math.random() * arr.length)];
}

export default function () {
    // Test 1: Create bet
    const createBetPayload = JSON.stringify({
        choice: randomChoice(choices),
        stake_amount: randomChoice(stakeAmounts),
        stake_token: 'SOL',
    });

    const createParams = {
        headers: {
            'Content-Type': 'application/json',
        },
    };

    const createStart = new Date();
    const createResponse = http.post(
        `${BACKEND_URL}/api/bets`,
        createBetPayload,
        createParams
    );
    betCreationDuration.add(new Date() - createStart);

    const createSuccess = check(createResponse, {
        'create bet: status is 200': (r) => r.status === 200,
        'create bet: has bet_id': (r) => {
            try {
                const body = JSON.parse(r.body);
                return body.bet && body.bet.bet_id;
            } catch {
                return false;
            }
        },
        'create bet: correct choice': (r) => {
            try {
                const body = JSON.parse(r.body);
                const requestData = JSON.parse(createBetPayload);
                return body.bet.choice === requestData.choice;
            } catch {
                return false;
            }
        },
    });

    if (!createSuccess) {
        errorRate.add(1);
        console.error(`Create bet failed: ${createResponse.status} - ${createResponse.body}`);
    } else {
        errorRate.add(0);

        // Test 2: Retrieve the bet we just created
        try {
            const createBody = JSON.parse(createResponse.body);
            const betId = createBody.bet.bet_id;

            const retrieveStart = new Date();
            const retrieveResponse = http.get(`${BACKEND_URL}/api/bets/${betId}`);
            betRetrievalDuration.add(new Date() - retrieveStart);

            const retrieveSuccess = check(retrieveResponse, {
                'get bet: status is 200': (r) => r.status === 200,
                'get bet: correct bet_id': (r) => {
                    try {
                        const body = JSON.parse(r.body);
                        return body.bet_id === betId;
                    } catch {
                        return false;
                    }
                },
            });

            if (!retrieveSuccess) {
                errorRate.add(1);
                console.error(`Retrieve bet failed: ${retrieveResponse.status}`);
            } else {
                errorRate.add(0);
            }
        } catch (e) {
            errorRate.add(1);
            console.error(`Failed to parse create response: ${e}`);
        }
    }

    // Test 3: List user bets (10% of requests)
    if (Math.random() < 0.1) {
        const listResponse = http.get(`${BACKEND_URL}/api/bets?user_wallet=TEST_WALLET&limit=20`);

        const listSuccess = check(listResponse, {
            'list bets: status is 200': (r) => r.status === 200,
            'list bets: returns array': (r) => {
                try {
                    const body = JSON.parse(r.body);
                    return Array.isArray(body);
                } catch {
                    return false;
                }
            },
        });

        errorRate.add(listSuccess ? 0 : 1);
    }

    // Test 4: Health check (5% of requests)
    if (Math.random() < 0.05) {
        const healthResponse = http.get(`${BACKEND_URL}/health`);

        const healthSuccess = check(healthResponse, {
            'health: status is 200': (r) => r.status === 200,
            'health: has status field': (r) => {
                try {
                    const body = JSON.parse(r.body);
                    return body.status === 'healthy';
                } catch {
                    return false;
                }
            },
        });

        errorRate.add(healthSuccess ? 0 : 1);
    }

    // Test 5: Invalid bet (error scenario - 5% of requests)
    if (Math.random() < 0.05) {
        const invalidPayload = JSON.stringify({
            choice: 'heads',
            stake_amount: 2_000_000_000_000, // Above maximum
            stake_token: 'SOL',
        });

        const errorResponse = http.post(
            `${BACKEND_URL}/api/bets`,
            invalidPayload,
            createParams
        );

        check(errorResponse, {
            'invalid bet: status is 400': (r) => r.status === 400,
            'invalid bet: has error object': (r) => {
                try {
                    const body = JSON.parse(r.body);
                    return body.error && body.error.code && body.error.category;
                } catch {
                    return false;
                }
            },
        });
    }

    // Test 6: Not found scenario (3% of requests)
    if (Math.random() < 0.03) {
        const fakeUuid = '00000000-0000-0000-0000-000000000000';
        const notFoundResponse = http.get(`${BACKEND_URL}/api/bets/${fakeUuid}`);

        check(notFoundResponse, {
            'not found: status is 404': (r) => r.status === 404,
            'not found: has error code': (r) => {
                try {
                    const body = JSON.parse(r.body);
                    return body.error && body.error.code === 'NOT_FOUND_BET';
                } catch {
                    return false;
                }
            },
        });
    }

    sleep(0.1); // Small delay between iterations
}

export function handleSummary(data) {
    return {
        'stdout': textSummary(data, { indent: ' ', enableColors: true }),
        'summary.json': JSON.stringify(data),
    };
}

function textSummary(data, options) {
    const indent = options.indent || '';
    const enableColors = options.enableColors || false;

    let output = '\n';
    output += `${indent}✅ Load Test Summary\n`;
    output += `${indent}${'='.repeat(60)}\n\n`;

    // Requests
    output += `${indent}📊 Requests:\n`;
    output += `${indent}  Total: ${data.metrics.http_reqs.values.count}\n`;
    output += `${indent}  Failed: ${data.metrics.http_req_failed.values.passes} (${(data.metrics.http_req_failed.values.rate * 100).toFixed(2)}%)\n\n`;

    // Duration
    output += `${indent}⏱️  Response Time:\n`;
    output += `${indent}  Avg: ${data.metrics.http_req_duration.values.avg.toFixed(2)}ms\n`;
    output += `${indent}  Min: ${data.metrics.http_req_duration.values.min.toFixed(2)}ms\n`;
    output += `${indent}  Max: ${data.metrics.http_req_duration.values.max.toFixed(2)}ms\n`;
    output += `${indent}  P95: ${data.metrics.http_req_duration.values['p(95)'].toFixed(2)}ms\n`;
    output += `${indent}  P99: ${data.metrics.http_req_duration.values['p(99)'].toFixed(2)}ms\n\n`;

    // Custom metrics
    if (data.metrics.errors) {
        output += `${indent}❌ Error Rate: ${(data.metrics.errors.values.rate * 100).toFixed(2)}%\n`;
    }
    if (data.metrics.bet_creation_duration) {
        output += `${indent}🎲 Bet Creation Avg: ${data.metrics.bet_creation_duration.values.avg.toFixed(2)}ms\n`;
    }
    if (data.metrics.bet_retrieval_duration) {
        output += `${indent}🔍 Bet Retrieval Avg: ${data.metrics.bet_retrieval_duration.values.avg.toFixed(2)}ms\n`;
    }

    output += '\n';
    return output;
}
