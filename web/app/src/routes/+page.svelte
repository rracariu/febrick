<script lang="ts">
	import * as Breadcrumb from '$lib/components/ui/breadcrumb/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Card from '$lib/components/ui/card/index';
	import Input from '$lib/components/ui/input/input.svelte';
	import * as Table from '$lib/components/ui/table/index.js';
	import * as Tabs from '$lib/components/ui/tabs/index';

	import { ArrowUpRight } from '@lucide/svelte';

	import { onMount } from 'svelte';

	import type { Brick, Curie } from 'febrick';
	import brickTtl from '../../../../Brick.ttl?raw';
	import Property from '$lib/components/Property.svelte';

	const rootClass = { prefix: 'brick', localName: 'Entity' } as Curie;

	let brick: Brick | undefined = $state(undefined);
	let path: Curie[] = $state([rootClass]);
	let search: string = $state('');
	let loadedClassNames: Curie[] = $state([]);

	const filteredCuries = $derived.by(() =>
		search.length
			? loadedClassNames.filter((entity) =>
					entity.localName.toLowerCase().startsWith(search.toLowerCase())
				)
			: loadedClassNames
	);

	onMount(async () => {
		const { Brick } = await import('febrick');
		brick = new Brick(brickTtl);
		subClassOf(rootClass);

		console.log('Ontology loaded');
	});

	function subClassOf(uri: Curie): void {
		if (!brick) {
			return;
		}

		const subClasses = brick.subClassOf(uri);
		loadedClassNames = subClasses.length ? subClasses : [uri];
	}

	function navigateToLevel(index: number): void {
		search = '';
		const part = path[index];
		path = path.slice(0, index + 1);
		subClassOf(part);
	}

	function navigateToClass(uri: Curie): void {
		search = '';

		if (path.find((p) => p.prefix === uri.prefix && p.localName === uri.localName)) {
			// already in path
			return;
		}

		path = [...path, uri];
		subClassOf(uri);
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
										><Button variant="link" onclick={() => navigateToLevel(index)}
											>{part.prefix}: {part.localName}</Button
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
					{#each filteredCuries as curie}
						{@const entity = brick?.classDescription(curie)}
						<Card.Root class="m-1"
							><Card.Header>
								<Card.Title>
									<div class="flex flex-row items-center justify-between">
										{curie.prefix}:{curie.localName}
										<Button
											variant="link"
											onclick={() => {
												navigateToClass(curie);
											}}
											><ArrowUpRight />
										</Button>
									</div>
								</Card.Title>

								<Card.Description class="overflow-hidden truncate"
									><span title={entity.definition}>{entity.definition}</span></Card.Description
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
										{#if entity.properties.length === 0}
											<Table.Row>
												<Table.Cell class="text-center">No properties</Table.Cell>
											</Table.Row>
										{:else}
											{#each entity.properties as prop}
												<Property property={prop} navigate={(curie) => navigateToClass(curie)} />
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
