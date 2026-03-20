module.exports = {
  apps: [
    {
      name: "repobox-landing",
      script: "node_modules/.bin/next",
      args: "start -p 3480",
      cwd: "/home/xiko/repobox-landing",
      env: {
        NODE_ENV: "production",
      },
    },
  ],
};
