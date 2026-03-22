import { NextRequest, NextResponse } from 'next/server';
import { runQuery, type Repo, type RepoPermission } from '@/lib/database';
import { getCommitCount, getContributorCount, getLastCommitDate, getReadmeFirstLine } from '@/lib/git';
import { ensurePermissionsScanned } from '@/lib/permissionScanner';

interface RepoWithMetadata extends Repo {
  commit_count: number;
  contributor_count: number;
  last_commit_date: string | null;
  description: string | null;
}

interface ContributorRepo extends RepoWithMetadata {
  permissions: string[];
}

function enrichRepo(repo: Repo): RepoWithMetadata {
  try {
    const commitCount = getCommitCount(repo.address, repo.name);
    const contributorCount = getContributorCount(repo.address, repo.name);
    const lastCommitDate = getLastCommitDate(repo.address, repo.name);
    const description = getReadmeFirstLine(repo.address, repo.name);

    return {
      ...repo,
      commit_count: commitCount,
      contributor_count: contributorCount,
      last_commit_date: lastCommitDate,
      description
    };
  } catch (error) {
    console.warn(`Error processing repo ${repo.address}/${repo.name}:`, error);
    return {
      ...repo,
      commit_count: 0,
      contributor_count: 0,
      last_commit_date: null,
      description: null
    };
  }
}

function sortRepos<T extends RepoWithMetadata>(repos: T[], sort: string): T[] {
  switch (sort) {
    case 'commits':
      return repos.sort((a, b) => b.commit_count - a.commit_count);
    case 'latest':
    default:
      return repos.sort((a, b) => {
        const dateA = a.last_commit_date ? new Date(a.last_commit_date).getTime() : 0;
        const dateB = b.last_commit_date ? new Date(b.last_commit_date).getTime() : 0;
        return dateB - dateA;
      });
  }
}

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sort = searchParams.get('sort') || 'latest';
    const limit = parseInt(searchParams.get('limit') || '50');
    const owner = searchParams.get('owner');

    if (owner) {
      // Ensure permissions are scanned for owner queries
      ensurePermissionsScanned();

      // 1. Get owned repos
      const ownedRepos = runQuery<Repo>(
        'SELECT * FROM repos WHERE LOWER(owner_address) = LOWER(?)',
        [owner]
      );

      const ownedWithMeta = ownedRepos.map(enrichRepo);

      // 2. Get repos where this address has explicit permissions or wildcard access
      const evmIdentity = `evm:${owner}`;
      const permEntries = runQuery<RepoPermission>(
        `SELECT * FROM repo_permissions WHERE LOWER(identity) = LOWER(?) OR identity = '*'`,
        [evmIdentity]
      );

      // Build a map of repo -> permissions, excluding repos already owned
      const ownedSet = new Set(ownedRepos.map(r => `${r.address}/${r.name}`.toLowerCase()));
      const contributorMap = new Map<string, Set<string>>();

      for (const perm of permEntries) {
        const key = `${perm.repo_address}/${perm.repo_name}`.toLowerCase();
        if (ownedSet.has(key)) continue;

        if (!contributorMap.has(key)) {
          contributorMap.set(key, new Set());
        }
        contributorMap.get(key)!.add(perm.verb);
      }

      // Fetch repo details for contributor repos
      const contributorRepos: ContributorRepo[] = [];
      for (const [key, verbs] of contributorMap) {
        const [address, name] = key.split('/');
        const repoRows = runQuery<Repo>(
          'SELECT * FROM repos WHERE LOWER(address) = LOWER(?) AND LOWER(name) = LOWER(?)',
          [address, name]
        );
        if (repoRows.length > 0) {
          const enriched = enrichRepo(repoRows[0]);
          contributorRepos.push({
            ...enriched,
            permissions: Array.from(verbs)
          });
        }
      }

      const sortedOwned = sortRepos(ownedWithMeta, sort).slice(0, limit);
      const sortedContributor = sortRepos(contributorRepos, sort).slice(0, limit);

      return NextResponse.json({
        repos: sortedOwned,
        contributor_repos: sortedContributor,
        total: ownedWithMeta.length,
        total_contributor: contributorRepos.length,
        sort,
        limit
      });
    }

    // No owner filter — return all repos
    const repos = runQuery<Repo>('SELECT * FROM repos');
    const reposWithMetadata = repos.map(enrichRepo);
    const sortedRepos = sortRepos(reposWithMetadata, sort).slice(0, limit);

    return NextResponse.json({
      repos: sortedRepos,
      total: reposWithMetadata.length,
      sort,
      limit
    });
  } catch (error) {
    console.error('Error fetching repos:', error);
    return NextResponse.json(
      { error: 'Failed to fetch repositories' },
      { status: 500 }
    );
  }
}
