declare const process: any;

export const environment = {
  production: true,
  wsUrl: process.env['BACKEND_WS_URL'],
  apiUrl: process.env['BACKEND_URL']
};