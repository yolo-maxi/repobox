module.exports = {
  apps: [
    {
      name: "repobox-landing",
      script: ".next/standalone/server.js",
      cwd: "/home/xiko/repobox-landing",
      env: {
        HOSTNAME: "0.0.0.0",
        NODE_ENV: "production",
        PORT: "3480",
      },
    },
  ],
};
