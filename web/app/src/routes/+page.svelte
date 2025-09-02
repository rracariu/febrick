<script lang="ts">
	import * as Breadcrumb from '$lib/components/ui/breadcrumb/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Card from '$lib/components/ui/card/index';
	import Input from '$lib/components/ui/input/input.svelte';
	import * as Table from '$lib/components/ui/table/index.js';
	import * as Tabs from '$lib/components/ui/tabs/index';

	import { ArrowUpRight } from '@lucide/svelte';

	import { onMount } from 'svelte';

	import Badge from '$lib/components/ui/badge/badge.svelte';
	import { Brick, type BrickProperty } from 'febrick';
	import brickTtl from '../../../../Brick.ttl?raw';

	const rootClass = 'Entity';

	let brick: Brick | undefined = $state(undefined);
	let path: string[] = $state([rootClass]);
	let search: string = $state('');
	let loadedClassNames: string[] = $state([]);

	const filteredClassNames = $derived.by(() =>
		search.length
			? loadedClassNames.filter((entity) => entity.toLowerCase().startsWith(search.toLowerCase()))
			: loadedClassNames
	);

	onMount(() => {
		brick = new Brick(brickTtl);
		subClassOf(rootClass);

		console.log('Ontology loaded');
	});

	function subClassOf(name: string): void {
		loadedClassNames = brick?.subClassOf(name) ?? [];
	}

	function navigateToLevel(index: number): void {
		search = '';
		const part = path[index];
		path = path.slice(0, index + 1);
		subClassOf(part);
	}

	function navigateToClass(name: string): void {
		search = '';
		path = [...path, name];
		subClassOf(name);
	}

	function constraints(prop: BrickProperty): string[] {
		if (prop.logicalConstraints.length) {
			const constraint = prop.logicalConstraints[0];
			if (!('Or' in constraint)) {
				return [];
			}

			return constraint.Or.map((prop) => prop.class);
		} else {
			return [];
		}
	}
</script>

<Tabs.Content value="classes" class="space-y-4">
	<Card.Root class="col-span-4">
		<Card.Header>
			<Card.Title
				><div class="flex flex-row">
					<div class="flex-1">
						<Breadcrumb.Root>
							<Breadcrumb.List>
								<Breadcrumb.Separator />
								{#each path as part, index}
									<Breadcrumb.Item
										><Button variant="link" onclick={() => navigateToLevel(index)}>{part}</Button
										></Breadcrumb.Item
									>
									{#if index < path.length - 1}
										<Breadcrumb.Separator />
									{/if}
								{/each}
							</Breadcrumb.List>
						</Breadcrumb.Root>
					</div>
					<div class="flex flex-row gap-2">
						<Input placeholder="Find class..." bind:value={search} />
					</div>
				</div></Card.Title
			>
		</Card.Header>
		<Card.Content>
			<div class="grid grid-cols-4">
				{#if brick}
					{#each filteredClassNames as className}
						{@const desc = brick?.classDescription(className)}
						<Card.Root class="m-1"
							><Card.Header>
								<Card.Title>
									<div class="flex flex-row items-center justify-between">
										{className}
										<Button
											variant="link"
											onclick={() => {
												navigateToClass(className);
											}}
											><ArrowUpRight />
										</Button>
									</div>
								</Card.Title>

								<Card.Description class="overflow-hidden truncate"
									><span title={desc.definition}>{desc.definition}</span></Card.Description
								>
							</Card.Header>
							<Card.Content>
								<Table.Root>
									<Table.Header>
										<Table.Row>
											<Table.Head>Path</Table.Head>
											<Table.Head>Class</Table.Head>
										</Table.Row>
									</Table.Header>
									<Table.Body>
										{#if desc.properties.length === 0}
											<Table.Row>
												<Table.Cell class="text-center">No properties</Table.Cell>
											</Table.Row>
										{:else}
											{#each desc.properties as prop}
												<Table.Row>
													<Table.Cell
														class="justify-self-end border-b-2 border-b-slate-50 align-text-top"
													>
														{prop.path}
													</Table.Cell>
													<Table.Cell
														class="justify-self-start border-b-2 border-b-slate-50 align-text-top"
													>
														{#if prop.class}
															<Button variant="link" onclick={() => navigateToClass(prop.class)}
																>{prop.class}</Button
															>
														{:else}
															{#each constraints(prop) as c, i}
																<div class="m-1 flex content-center items-center">
																	{#if i === 0}
																		<Badge variant="outline" class="mr-1">One of:</Badge>
																	{/if}
																	<Button variant="link" onclick={() => navigateToClass(c)}
																		>{c}</Button
																	>
																</div>
															{/each}
														{/if}
													</Table.Cell>
												</Table.Row>
											{/each}
										{/if}
									</Table.Body>
								</Table.Root>
							</Card.Content>
						</Card.Root>
					{/each}
				{/if}
			</div>
		</Card.Content>
	</Card.Root>
</Tabs.Content>
