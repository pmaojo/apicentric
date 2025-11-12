import type { Log } from './types';

// The 'services' data is now fetched from the API route.
// This file is kept for the 'logs' data for now.

export const logs: Log[] = [
    { id: '1', timestamp: new Date(Date.now() - 5000).toISOString(), service: 'User Management', method: 'GET', route: '/users', status: 200, ip: '192.168.1.10' },
    { id: '2', timestamp: new Date(Date.now() - 10000).toISOString(), service: 'Product Catalog', method: 'GET', route: '/products/prod_123', status: 200, ip: '192.168.1.12' },
    { id: '3', timestamp: new Date(Date.now() - 15000).toISOString(), service: 'User Management', method: 'POST', route: '/users', status: 201, ip: '192.168.1.10' },
    { id: '4', timestamp: new Date(Date.now() - 20000).toISOString(), service: 'Authentication', method: 'POST', route: '/auth/login', status: 401, ip: '10.0.0.5' },
    { id: '5', timestamp: new Date(Date.now() - 25000).toISOString(), service: 'Product Catalog', method: 'GET', route: '/products', status: 200, ip: '192.168.1.12' },
    { id: '6', timestamp: new Date(Date.now() - 30000).toISOString(), service: 'Order Processing', method: 'POST', route: '/orders', status: 500, ip: '172.16.0.88' },
    { id: '7', timestamp: new Date(Date.now() - 35000).toISOString(), service: 'User Management', method: 'GET', route: '/users/usr_456', status: 404, ip: '192.168.1.15' },
    { id: '8', timestamp: new Date(Date.now() - 40000).toISOString(), service: 'Order Processing', method: 'GET', route: '/orders/ord_789', status: 200, ip: '172.16.0.90' },
    { id: '9', timestamp: new Date(Date.now() - 45000).toISOString(), service: 'User Management', method: 'DELETE', route: '/users/usr_123', status: 204, ip: '192.168.1.10' },
    { id: '10', timestamp: new Date(Date.now() - 50000).toISOString(), service: 'Product Catalog', method: 'GET', route: '/products/prod_999', status: 404, ip: '192.168.1.22' },
];
